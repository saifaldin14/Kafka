//! Strategies for values that hold together: partition watermarks in order, a
//! producer's batch stream, and topic-partition sets whose topic names collide.
//!
//! Each comes with its invariant stated, and where a test needs the broken case too, an
//! `invalid_` strategy that breaks it in one labelled way. Storage, replication and
//! transaction tests build their scenarios from these instead of filtering random
//! values.

use std::collections::BTreeSet;

use proptest::collection::vec;
use proptest::prelude::{Just, Strategy, prop_oneof};

use super::{
    colliding_topic_names, offset, partition_index, producer_epoch, producer_id, sequence,
    topic_name,
};
use crate::{Offset, ProducerEpoch, ProducerId, Sequence, TopicPartition};

/// A partition's three watermarks. Valid when
/// `log_start_offset <= high_watermark <= log_end_offset`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Watermarks {
    /// The first offset still in the log.
    pub log_start_offset: Offset,
    /// The offset below which every record is replicated to the ISR.
    pub high_watermark: Offset,
    /// The offset the next record will get.
    pub log_end_offset: Offset,
}

/// Valid [`Watermarks`], biased towards the equalities: an empty log, a fully
/// replicated log and a log with nothing deleted.
pub fn watermarks() -> impl Strategy<Value = Watermarks> {
    let ordered = |mut offsets: [Offset; 3]| {
        offsets.sort();
        let [log_start_offset, high_watermark, log_end_offset] = offsets;
        Watermarks {
            log_start_offset,
            high_watermark,
            log_end_offset,
        }
    };
    prop_oneof![
        1 => offset().prop_map(move |at| ordered([at, at, at])),
        1 => (offset(), offset()).prop_map(move |(a, b)| ordered([a, b, b])),
        1 => (offset(), offset()).prop_map(move |(a, b)| ordered([a, a, b])),
        7 => (offset(), offset(), offset()).prop_map(move |(a, b, c)| ordered([a, b, c])),
    ]
}

/// [`Watermarks`] that break the order: the high watermark below the log start offset,
/// or above the log end offset.
pub fn invalid_watermarks() -> impl Strategy<Value = Watermarks> {
    (offset(), offset(), offset(), proptest::bool::ANY).prop_filter_map(
        "two distinct offsets",
        |(a, b, other, below_start)| {
            let (low, high) = (a.min(b), a.max(b));
            (low != high).then(|| {
                if below_start {
                    Watermarks {
                        log_start_offset: high,
                        high_watermark: low,
                        log_end_offset: high.max(other),
                    }
                } else {
                    Watermarks {
                        log_start_offset: low.min(other),
                        high_watermark: high,
                        log_end_offset: low,
                    }
                }
            })
        },
    )
}

/// One batch an idempotent producer appends to a partition: the sequences of its first
/// and last records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProducerBatch {
    /// The producer, the same for every batch of a stream.
    pub producer_id: ProducerId,
    /// The producer's epoch when it wrote the batch.
    pub producer_epoch: ProducerEpoch,
    /// The sequence of the batch's first record.
    pub first_sequence: Sequence,
    /// The sequence of its last record.
    pub last_sequence: Sequence,
}

/// How [`invalid_producer_batches`] breaks the last batch, named after the check in
/// Kafka's `ProducerAppendInfo` that rejects it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProducerBatchViolation {
    /// Same epoch, but the first sequence skips past the next one (`OutOfOrderSequence`).
    SequenceGap,
    /// A new epoch whose first sequence isn't 0 (`OutOfOrderSequence`).
    NewEpochNotAtZero,
    /// An epoch lower than the last one (`InvalidProducerEpoch`).
    EpochDecreased,
}

/// One step of a generated stream.
#[derive(Debug, Clone, Copy)]
struct Step {
    bump_epoch: bool,
    records: u32,
}

fn steps() -> impl Strategy<Value = Vec<Step>> {
    let step = (proptest::bool::weighted(0.2), 1..=5_u32).prop_map(|(bump_epoch, records)| Step {
        bump_epoch,
        records,
    });
    vec(step, 1..=12)
}

/// The batches `steps` describe, starting at `epoch` and `first`.
fn stream(
    id: ProducerId,
    epoch: ProducerEpoch,
    first: Sequence,
    steps: &[Step],
) -> Vec<ProducerBatch> {
    let mut batches: Vec<ProducerBatch> = Vec::with_capacity(steps.len());
    for step in steps {
        let (producer_epoch, first_sequence) = match batches.last() {
            None => (epoch, first),
            Some(last) => match last
                .producer_epoch
                .checked_bump()
                .filter(|_| step.bump_epoch)
            {
                Some(bumped) => (bumped, Sequence::ZERO),
                None => (last.producer_epoch, last.last_sequence.wrapping_add(1)),
            },
        };
        batches.push(ProducerBatch {
            producer_id: id,
            producer_epoch,
            first_sequence,
            last_sequence: first_sequence.wrapping_add(step.records - 1),
        });
    }
    batches
}

/// Batch streams Kafka accepts from one idempotent producer: each batch continues the
/// last one's sequence in the same epoch (wrapping from `i32::MAX` to 0), or starts a
/// higher epoch at sequence 0. The first batch may start anywhere, as it may for a
/// producer the partition has no state for. Streams start near the wrap point and the
/// epoch's exhaustion often.
// Parity: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java@4.4.0 ProducerAppendInfo.checkSequence
pub fn producer_batches() -> impl Strategy<Value = Vec<ProducerBatch>> {
    (producer_id(), producer_epoch(), sequence(), steps())
        .prop_map(|(id, epoch, first, steps)| stream(id, epoch, first, &steps))
}

/// A valid stream from [`producer_batches`] followed by one batch Kafka rejects, and how
/// it breaks the rules. A violation the stream's state makes impossible (a lower epoch
/// after epoch 0, a higher one after `i16::MAX`) becomes a sequence gap instead.
pub fn invalid_producer_batches()
-> impl Strategy<Value = (Vec<ProducerBatch>, ProducerBatchViolation)> {
    let violation = prop_oneof![
        Just(ProducerBatchViolation::SequenceGap),
        Just(ProducerBatchViolation::NewEpochNotAtZero),
        Just(ProducerBatchViolation::EpochDecreased),
    ];
    // Streams always have a batch, so nothing is filtered out.
    (producer_batches(), violation, 1..=5_u32, 1..=5_u32).prop_filter_map(
        "a non-empty stream",
        |(mut batches, wanted, skip, records)| {
            let &last = batches.last()?;
            let gap = ProducerBatch {
                first_sequence: last.last_sequence.wrapping_add(1 + skip),
                last_sequence: last.last_sequence.wrapping_add(skip + records),
                ..last
            };
            let (batch, violation) = match wanted {
                ProducerBatchViolation::NewEpochNotAtZero => {
                    match last.producer_epoch.checked_bump() {
                        Some(bumped) => {
                            let first = Sequence::ZERO.wrapping_add(skip);
                            (
                                ProducerBatch {
                                    producer_epoch: bumped,
                                    first_sequence: first,
                                    last_sequence: first.wrapping_add(records - 1),
                                    ..last
                                },
                                wanted,
                            )
                        }
                        None => (gap, ProducerBatchViolation::SequenceGap),
                    }
                }
                ProducerBatchViolation::EpochDecreased => {
                    match ProducerEpoch::try_from(last.producer_epoch.get() - 1) {
                        Ok(lower) => (
                            ProducerBatch {
                                producer_epoch: lower,
                                first_sequence: Sequence::ZERO,
                                last_sequence: Sequence::ZERO.wrapping_add(records - 1),
                                ..last
                            },
                            wanted,
                        ),
                        Err(_) => (gap, ProducerBatchViolation::SequenceGap),
                    }
                }
                ProducerBatchViolation::SequenceGap => (gap, wanted),
            };
            batches.push(batch);
            Some((batches, violation))
        },
    )
}

/// Sets of topic-partitions that include partitions of two different topics whose names
/// collide (they differ only in `.` and `_`), among up to two other topics.
pub fn colliding_topic_partitions() -> impl Strategy<Value = BTreeSet<TopicPartition>> {
    let partitions = || vec(partition_index(), 1..=3);
    (
        colliding_topic_names().prop_filter("different names", |(a, b)| a != b),
        partitions(),
        partitions(),
        vec((topic_name(), partition_index()), 0..=2),
    )
        .prop_map(
            |((first, second), first_partitions, second_partitions, others)| {
                let mut set = BTreeSet::new();
                for (topic, partitions) in [(first, first_partitions), (second, second_partitions)]
                {
                    for partition in partitions {
                        set.insert(TopicPartition {
                            topic: topic.clone(),
                            partition,
                        });
                    }
                }
                set.extend(
                    others
                        .into_iter()
                        .map(|(topic, partition)| TopicPartition { topic, partition }),
                );
                set
            },
        )
}

#[cfg(test)]
mod tests {
    use proptest::prelude::{prop_assert, prop_assert_eq, proptest};

    use super::{
        ProducerBatch, ProducerBatchViolation, colliding_topic_partitions,
        invalid_producer_batches, invalid_watermarks, producer_batches, watermarks,
    };

    /// Kafka's checks on an idempotent producer's next batch, given the last one (none
    /// for a producer the partition has no state for).
    // Parity: storage/src/main/java/org/apache/kafka/storage/internals/log/ProducerAppendInfo.java@4.4.0 ProducerAppendInfo.checkProducerEpoch, ProducerAppendInfo.checkSequence, ProducerAppendInfo.inSequence
    fn check(
        last: Option<&ProducerBatch>,
        next: &ProducerBatch,
    ) -> Result<(), ProducerBatchViolation> {
        let Some(last) = last else {
            return Ok(());
        };
        if next.producer_epoch < last.producer_epoch {
            return Err(ProducerBatchViolation::EpochDecreased);
        }
        if next.producer_epoch != last.producer_epoch {
            return match next.first_sequence.get() {
                0 => Ok(()),
                _ => Err(ProducerBatchViolation::NewEpochNotAtZero),
            };
        }
        let (last_sequence, first) = (last.last_sequence.get(), next.first_sequence.get());
        let in_sequence = i64::from(first) == i64::from(last_sequence) + 1
            || (first == 0 && last_sequence == i32::MAX);
        if in_sequence {
            Ok(())
        } else {
            Err(ProducerBatchViolation::SequenceGap)
        }
    }

    fn first_violation(batches: &[ProducerBatch]) -> Option<(usize, ProducerBatchViolation)> {
        batches.iter().enumerate().find_map(|(index, batch)| {
            let last = index
                .checked_sub(1)
                .and_then(|previous| batches.get(previous));
            check(last, batch).err().map(|violation| (index, violation))
        })
    }

    proptest! {
        #[test]
        fn watermarks_are_ordered(marks in watermarks()) {
            prop_assert!(marks.log_start_offset <= marks.high_watermark);
            prop_assert!(marks.high_watermark <= marks.log_end_offset);
        }

        #[test]
        fn invalid_watermarks_are_out_of_order(marks in invalid_watermarks()) {
            prop_assert!(
                marks.high_watermark < marks.log_start_offset
                    || marks.log_end_offset < marks.high_watermark
            );
        }

        #[test]
        fn producer_batches_pass_kafkas_checks(batches in producer_batches()) {
            prop_assert_eq!(first_violation(&batches), None);
            let id = batches[0].producer_id;
            prop_assert!(batches.iter().all(|batch| batch.producer_id == id));
        }

        #[test]
        fn invalid_producer_batches_fail_on_the_last_batch_as_labelled(
            (batches, violation) in invalid_producer_batches()
        ) {
            prop_assert_eq!(first_violation(&batches), Some((batches.len() - 1, violation)));
        }

        #[test]
        fn colliding_topic_partitions_contain_a_collision(partitions in colliding_topic_partitions()) {
            let collides = partitions.iter().any(|a| {
                partitions.iter().any(|b| a.topic != b.topic && a.topic.collides_with(&b.topic))
            });
            prop_assert!(collides, "{partitions:?}");
        }
    }
}
