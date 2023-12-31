use crate::{CliResult, DetermineAccountId, Error, Output};
use clap::Subcommand;
use divviup_client::{DivviupClient, Histogram, NewTask, Uuid, Vdaf};
use humantime::Duration;

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum VdafName {
    Count,
    Histogram,
    Sum,
    CountVec,
    SumVec,
}

#[derive(Subcommand, Debug)]
pub enum TaskAction {
    /// list all tasks for the target account
    List,

    /// create a new task for the target account
    Create {
        #[arg(long)]
        name: String,
        #[arg(long)]
        leader_aggregator_id: Uuid,
        #[arg(long)]
        helper_aggregator_id: Uuid,
        #[arg(long)]
        vdaf: VdafName,
        #[arg(long)]
        min_batch_size: u64,
        #[arg(long)]
        max_batch_size: Option<u64>,
        #[arg(long)]
        time_precision: Duration,
        #[arg(long)]
        collector_credential_id: Uuid,
        #[arg(long, value_delimiter = ',')]
        categorical_buckets: Option<Vec<String>>,
        #[arg(long, value_delimiter = ',')]
        continuous_buckets: Option<Vec<u64>>,
        #[arg(long, required_if_eq_any([("vdaf", "count_vec"), ("vdaf", "sum_vec")]))]
        length: Option<u64>,
        #[arg(long, required_if_eq_any([("vdaf", "sum"), ("vdaf", "sum_vec")]))]
        bits: Option<u8>,
        #[arg(long)]
        chunk_length: Option<u64>,
    },

    /// rename a task
    Rename { task_id: String, name: String },

    /// retrieve the collector auth tokens for a task
    CollectorAuthTokens { task_id: String },
}

impl TaskAction {
    pub(crate) async fn run(
        self,
        account_id: DetermineAccountId,
        client: DivviupClient,
        output: Output,
    ) -> CliResult {
        let account_id = account_id.await?;

        match self {
            TaskAction::List => output.display(client.tasks(account_id).await?),
            TaskAction::Create {
                name,
                leader_aggregator_id,
                helper_aggregator_id,
                vdaf,
                min_batch_size,
                max_batch_size,
                time_precision,
                collector_credential_id,
                categorical_buckets,
                continuous_buckets,
                length,
                bits,
                chunk_length,
            } => {
                let vdaf = match vdaf {
                    VdafName::Count => Vdaf::Count,
                    VdafName::Histogram => {
                        match (length, categorical_buckets, continuous_buckets) {
                            (Some(length), None, None) => Vdaf::Histogram(Histogram::Length {
                                length,
                                chunk_length,
                            }),
                            (None, Some(buckets), None) => {
                                Vdaf::Histogram(Histogram::Categorical {
                                    buckets,
                                    chunk_length,
                                })
                            }
                            (None, None, Some(buckets)) => Vdaf::Histogram(Histogram::Continuous {
                                buckets,
                                chunk_length,
                            }),
                            (None, None, None) => {
                                return Err(Error::Other("continuous-buckets, categorical-buckets, or length are required for histogram vdaf".into()));
                            }
                            _ => {
                                return Err(Error::Other("continuous-buckets, categorical-buckets, and length are mutually exclusive".into()));
                            }
                        }
                    }
                    VdafName::Sum => Vdaf::Sum {
                        bits: bits.unwrap(),
                    },
                    VdafName::CountVec => Vdaf::CountVec {
                        length: length.unwrap(),
                        chunk_length,
                    },
                    VdafName::SumVec => Vdaf::SumVec {
                        bits: bits.unwrap(),
                        length: length.unwrap(),
                        chunk_length,
                    },
                };

                let time_precision_seconds = time_precision.as_secs();

                let task = NewTask {
                    name,
                    leader_aggregator_id,
                    helper_aggregator_id,
                    vdaf,
                    min_batch_size,
                    max_batch_size,
                    time_precision_seconds,
                    collector_credential_id,
                };

                output.display(client.create_task(account_id, task).await?)
            }

            TaskAction::Rename { task_id, name } => {
                output.display(client.rename_task(&task_id, &name).await?)
            }

            TaskAction::CollectorAuthTokens { task_id } => {
                output.display(client.task_collector_auth_tokens(&task_id).await?)
            }
        }

        Ok(())
    }
}
