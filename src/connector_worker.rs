use std::sync::Arc;

use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;

use crate::{
    app_state::{ActionKind, ConnectorCommands, ConnectorEvents},
    settings::connectors::ConfiguredConnector,
};

pub struct ConnectorWorker {
    connector: Arc<ConfiguredConnector>,
    cancellation_token: CancellationToken,
}

impl ConnectorWorker {
    pub fn new(connector: Arc<ConfiguredConnector>, cancellation_token: CancellationToken) -> Self {
        Self {
            connector,
            cancellation_token,
        }
    }

    pub async fn run(
        &self,
        mut command_rx: mpsc::Receiver<ConnectorCommands>,
        event_tx: mpsc::Sender<ConnectorEvents>,
    ) -> JoinHandle<()> {
        let connector = Arc::clone(&self.connector);
        let cancellation_token = self.cancellation_token.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(connector.update_interval);
            let connector = &connector.connector;
            let connector_name = connector.name();
            loop {
                tokio::select! {
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                    _ = interval.tick() => {
                        match connector.get_torrent_list().await {
                            Ok(torrents) => {
                                // TODO: handle Result
                                let _ = event_tx.send(ConnectorEvents::UpdateTorrentList(Arc::clone( &connector_name ), torrents)).await;
                            },
                            Err(e) => {
                                // TODO: handle Result
                                let _ = event_tx.send(ConnectorEvents::Error(e)).await;
                            }
                        }
                    }
                    Some(command) = command_rx.recv() => {
                        match command {
                            ConnectorCommands::Add(source) => {
                                match connector.add_torrent(source).await {
                                    Ok(_) => {
                                        // TODO: handle Result
                                        let _ = event_tx.send(ConnectorEvents::AddOk).await;
                                    },
                                    Err(e) => {
                                        // TODO: handle Result
                                        let _ = event_tx.send(ConnectorEvents::Error(e)).await;
                                    },
                                }
                            },
                            ConnectorCommands::Action {kind, info_hash} => {
                                match kind {
                                    ActionKind::Pause => {
                                        match connector.pause_torrent(info_hash).await {
                                            Ok(_) => {
                                                // TODO: handle Result
                                                let _ = event_tx.send(ConnectorEvents::PauseOk).await;
                                            },
                                            Err(e) => {
                                                // TODO: handle Result
                                                let _ = event_tx.send(ConnectorEvents::Error(e)).await;
                                            }
                                        }
                                    },
                                    ActionKind::Start => {
                                        match connector.start_torrent(info_hash).await {
                                            Ok(_) => {
                                                // TODO: handle Result
                                                let _ = event_tx.send(ConnectorEvents::StartOk).await;
                                            },
                                            Err(e) => {
                                                // TODO: handle Result
                                                let _ = event_tx.send(ConnectorEvents::Error(e)).await;
                                            }
                                        }
                                    },
                                    ActionKind::Forget => {
                                        match connector.forget_torrent(info_hash).await {
                                            Ok(_) => {
                                                // TODO: handle Result
                                                let _  = event_tx.send(ConnectorEvents::ForgetOk).await;
                                            },
                                            Err(e) => {
                                                // TODO: handle Result
                                                let _ = event_tx.send(ConnectorEvents::Error(e)).await;
                                            },
                                        }
                                    },
                                    ActionKind::Delete => {
                                        match connector.delete_torrent(info_hash).await {
                                            Ok(_) => {
                                                // TODO: handle Result
                                                let _ = event_tx.send(ConnectorEvents::DeleteOk).await;
                                            },
                                            Err(e) => {
                                                // TODO: handle Result
                                                let _ = event_tx.send(ConnectorEvents::Error(e)).await;
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use assert_matches::assert_matches;
    use color_eyre::eyre::OptionExt;
    use fake::{Fake, Faker};
    use mockall::predicate;
    use pretty_assertions::assert_eq;
    use tokio::{sync::mpsc, task::JoinHandle};
    use tokio_util::sync::CancellationToken;

    use crate::{
        app_state::{ActionKind, ConnectorCommands, ConnectorEvents},
        connector_worker::ConnectorWorker,
        connectors::MockConnector,
        settings::connectors::ConfiguredConnector,
        torrent::{self, InfoHash, TorrentInfo},
    };

    struct Helper {
        event: ConnectorEvents,
        worker_handle: JoinHandle<()>,
        cancellation_token: CancellationToken,
    }
    impl Helper {
        fn worker_shutdown(&self) {
            self.cancellation_token.cancel();
        }
    }
    async fn setup(command: ConnectorCommands) -> color_eyre::Result<Helper> {
        let mut mock_connector = MockConnector::new();

        match command.clone() {
            ConnectorCommands::Add(source) => {
                let mut mock_connector = MockConnector::new();
                let source_clone = source.clone();
                mock_connector
                    .expect_add_torrent()
                    .with(predicate::eq(source))
                    .times(1)
                    .returning(|_| Ok(()));

                let expected_connector_name = String::from("test_connector");
                mock_connector
                    .expect_name()
                    .return_const(expected_connector_name.clone());

                let (command_tx, command_rx) = mpsc::channel(10);
                let (event_tx, mut event_rx) = mpsc::channel(10);

                let connector = Arc::new(ConfiguredConnector {
                    connector: Box::new(mock_connector),
                    update_interval: Duration::from_secs(120),
                });

                let cancellation_token = CancellationToken::new();
                let worker = ConnectorWorker::new(connector, cancellation_token.clone());

                let worker_handle = worker.run(command_rx, event_tx).await;

                command_tx
                    .send(ConnectorCommands::Add(source_clone))
                    .await?;

                let event = tokio::time::timeout(Duration::from_millis(100), event_rx.recv())
                    .await?
                    .ok_or_eyre("Channel closed")?;
                Ok(Helper {
                    event,
                    worker_handle,
                    cancellation_token,
                })
            }
            ConnectorCommands::Action { info_hash, kind } => {
                match kind {
                    ActionKind::Pause => {
                        mock_connector
                            .expect_pause_torrent()
                            .with(predicate::eq(info_hash))
                            .times(1)
                            .returning(|_| Ok(()));
                    }
                    ActionKind::Start => {
                        mock_connector
                            .expect_start_torrent()
                            .with(predicate::eq(info_hash))
                            .times(1)
                            .returning(|_| Ok(()));
                    }
                    ActionKind::Forget => {
                        mock_connector
                            .expect_forget_torrent()
                            .with(predicate::eq(info_hash))
                            .times(1)
                            .returning(|_| Ok(()));
                    }
                    ActionKind::Delete => {
                        mock_connector
                            .expect_delete_torrent()
                            .with(predicate::eq(info_hash))
                            .times(1)
                            .returning(|_| Ok(()));
                    }
                }
                mock_connector
                    .expect_get_torrent_list()
                    .returning(|| Ok(vec![Faker.fake::<TorrentInfo>()]));
                mock_connector
                    .expect_name()
                    .times(1)
                    .return_const(String::from("test_connector"));

                let connector = Arc::new(ConfiguredConnector {
                    connector: Box::new(mock_connector),
                    update_interval: Duration::from_secs(120),
                });

                let cancellation_token = CancellationToken::new();
                let worker = ConnectorWorker::new(connector, cancellation_token.clone());

                let (command_tx, command_rx) = mpsc::channel(10);
                let (event_tx, mut event_rx) = mpsc::channel(10);

                let worker_handle = worker.run(command_rx, event_tx).await;

                command_tx.send(command).await?;

                let event = tokio::time::timeout(Duration::from_millis(100), event_rx.recv())
                    .await?
                    .ok_or_eyre("Channel closed")?;

                Ok(Helper {
                    event,
                    worker_handle,
                    cancellation_token,
                })
            }
        }
    }

    #[tokio::test]
    async fn worker_handle_add_torrent_command() -> color_eyre::Result<()> {
        let magnet = Faker.fake::<torrent::Magnet>();
        let source = torrent::Source::Magnet(magnet);

        let command = ConnectorCommands::Add(source);

        let helper = setup(command).await?;

        helper.worker_shutdown();

        assert_matches!(helper.event, ConnectorEvents::AddOk);

        tokio::time::timeout(Duration::from_millis(50), helper.worker_handle).await??;

        Ok(())
    }

    #[tokio::test]
    async fn worker_handle_pause_torrent_command() -> color_eyre::Result<()> {
        let command = ConnectorCommands::Action {
            kind: ActionKind::Pause,
            info_hash: Faker.fake::<InfoHash>(),
        };

        let helper = setup(command).await?;

        helper.worker_shutdown();

        assert_matches!(helper.event, ConnectorEvents::PauseOk);

        tokio::time::timeout(Duration::from_millis(50), helper.worker_handle).await??;

        Ok(())
    }

    #[tokio::test]
    async fn worker_handle_start_torrent_command() -> color_eyre::Result<()> {
        let command = ConnectorCommands::Action {
            kind: ActionKind::Start,
            info_hash: Faker.fake::<InfoHash>(),
        };

        let helper = setup(command).await?;

        helper.worker_shutdown();

        assert_matches!(helper.event, ConnectorEvents::StartOk);

        tokio::time::timeout(Duration::from_millis(50), helper.worker_handle).await??;

        Ok(())
    }

    #[tokio::test]
    async fn worker_handle_forget_torrent_command() -> color_eyre::Result<()> {
        let command = ConnectorCommands::Action {
            kind: ActionKind::Forget,
            info_hash: Faker.fake::<InfoHash>(),
        };
        let helper = setup(command).await?;

        helper.worker_shutdown();
        assert_matches!(helper.event, ConnectorEvents::ForgetOk);

        tokio::time::timeout(Duration::from_millis(50), helper.worker_handle).await??;

        Ok(())
    }

    #[tokio::test]
    async fn worker_handle_delete_torrent_command() -> color_eyre::Result<()> {
        let info_hash = Faker.fake::<torrent::InfoHash>();
        let command = ConnectorCommands::Action {
            kind: ActionKind::Delete,
            info_hash,
        };

        let helper = setup(command).await?;

        helper.worker_shutdown();

        assert_matches!(helper.event, ConnectorEvents::DeleteOk);

        tokio::time::timeout(Duration::from_millis(50), helper.worker_handle).await??;

        Ok(())
    }

    #[tokio::test]
    async fn worker_update_torrent_list() -> color_eyre::Result<()> {
        let mut mock_connector = MockConnector::new();

        let torrent_info = Faker.fake::<TorrentInfo>();
        let expected_torrent_info = torrent_info.clone();
        mock_connector
            .expect_get_torrent_list()
            .times(1)
            .returning(move || Ok(vec![torrent_info.clone()]));

        let expected_connector_name = String::from("test_connector");
        mock_connector
            .expect_name()
            .return_const(expected_connector_name.clone());

        let (_command_tx, command_rx) = mpsc::channel::<ConnectorCommands>(10);
        let (event_tx, mut event_rx) = mpsc::channel::<ConnectorEvents>(10);

        let connector = Arc::new(ConfiguredConnector {
            connector: Box::new(mock_connector),
            update_interval: Duration::from_millis(50),
        });

        let cancellation_token = CancellationToken::new();
        let worker = ConnectorWorker::new(connector, cancellation_token.clone());

        let worker_handle = worker.run(command_rx, event_tx).await;
        let event = tokio::time::timeout(Duration::from_millis(50), event_rx.recv())
            .await?
            .ok_or_eyre("Channel closed")?;

        cancellation_token.cancel();

        match event {
            ConnectorEvents::UpdateTorrentList(connector_name, torrents) => {
                assert_eq!(connector_name, expected_connector_name.into());
                assert_eq!(torrents.len(), 1);
                assert_eq!(torrents[0].info_hash, expected_torrent_info.info_hash);
            }
            _ => panic!("Expected UpdateTorrentList event"),
        }

        tokio::time::timeout(Duration::from_millis(50), worker_handle).await??;
        Ok(())
    }

    // TODO: test connector errors
}
