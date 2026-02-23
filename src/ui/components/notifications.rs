use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    layout::{HorizontalAlignment, Rect},
    widgets::{Paragraph, Widget},
};

use crate::{
    action::ActionError,
    connectors::ConnectorEvents,
    settings::{Settings, styles::StyleMode},
    ui::Drawable,
};

pub enum Notification {
    Info(String),
    Error(String),
}
impl Notification {
    pub fn info(message: impl Into<String>) -> Self {
        let message = message.into();
        Notification::Info(message)
    }
    pub fn error(message: impl Into<String>) -> Self {
        let message = message.into();
        Notification::Error(message)
    }
}
impl From<ConnectorEvents> for Option<Notification> {
    fn from(value: ConnectorEvents) -> Self {
        match value {
            ConnectorEvents::AddOk => Some(Notification::Info("Torrent added".into())),
            ConnectorEvents::PauseOk => Some(Notification::Info("Torrent paused".into())),
            ConnectorEvents::StartOk => Some(Notification::Info("Torrent started".into())),
            ConnectorEvents::ForgetOk => Some(Notification::Info(
                "Torrent removed from list (files kept)".into(),
            )),
            ConnectorEvents::DeleteOk => {
                Some(Notification::Info("Torrent deleted with files".into()))
            }
            ConnectorEvents::Error(e) => Some(Notification::Error(e.to_string())),
            _ => None,
        }
    }
}
impl From<ActionError> for Option<Notification> {
    fn from(value: ActionError) -> Self {
        use ActionError::*;
        match value {
            CommandSendFailed { .. } => Some(Notification::error("Command send failed")),
            ConnectorNotFound => Some(Notification::error("Connector not found")),
            GetActionFailed { .. } => Some(Notification::error("Get action failed")),
            SendError { .. } => Some(Notification::error("Send error")),
            CreateMagnetError { .. } => Some(Notification::error("Create magnet error")),
            PlayError { .. } => Some(Notification::error("Play failed")),
            ClipboardInitError { .. } => {
                Some(Notification::error("Clipboard initialization error"))
            }
            GetFromClipboardError { .. } => {
                Some(Notification::error("Faied to get text from clipboard"))
            }
        }
    }
}

pub struct Notifications {
    notification: Option<Notification>,
    last_interaction: Option<Instant>,
}

impl Drawable for Notifications {
    fn draw(&mut self, buf: &mut Buffer, area: Rect, settings: &Settings) {
        if let Some(notification) = &self.notification {
            let timeout = settings.notification_timeout_millis;
            if let Some(last_interaction) = self.last_interaction
                && Instant::now().duration_since(last_interaction) > Duration::from_millis(timeout)
            {
                self.notification = None;
                self.last_interaction = None;
                return;
            }
            let info_style = settings.styles.get_style(&StyleMode::Notification, "info");
            let err_style = settings.styles.get_style(&StyleMode::Notification, "error");
            let msg = match notification {
                Notification::Info(m) => Paragraph::new(m.clone()).style(info_style),
                Notification::Error(e) => Paragraph::new(e.clone()).style(err_style),
            };

            Widget::render(msg.alignment(HorizontalAlignment::Right), area, buf);
        }
    }
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            notification: None,
            last_interaction: None,
        }
    }

    pub fn notify(&mut self, n: impl Into<Option<Notification>>) {
        self.notification = n.into();
    }

    pub fn on_user_interaction(&mut self) {
        if self.notification.is_some() {
            self.last_interaction = Some(Instant::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Drawable;
    use ratatui::{
        Terminal,
        backend::TestBackend,
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style},
    };

    use pretty_assertions::assert_eq;

    use crate::{
        connectors::ConnectorEvents,
        settings::Settings,
        ui::{Notifications, notifications::Notification},
    };

    struct TestHelper {
        terminal: Terminal<TestBackend>,
        component: Notifications,
    }

    impl TestHelper {
        fn new(width: u16, height: u16) -> color_eyre::Result<Self> {
            let backend = TestBackend::new(width, height);
            let terminal = Terminal::new(backend)?;

            let component = Notifications::new();
            Ok(Self {
                terminal,
                component,
            })
        }
        fn buffer(&self) -> &Buffer {
            self.terminal.backend().buffer()
        }
        fn draw(&mut self, settings: &Settings) -> color_eyre::Result<()> {
            self.terminal.draw(|frame| {
                let area = frame.area();
                let buffer = frame.buffer_mut();
                self.component.draw(buffer, area, settings);
            })?;
            Ok(())
        }
        fn notify(&mut self, n: impl Into<Option<Notification>>) {
            self.component.notify(n);
        }
    }

    #[test]
    fn notify_add_ok() -> color_eyre::Result<()> {
        let config_str = r#"
            [styles.Notification]
            info = "blue on rgb:0,0,0"
            error = "red on blue"
        "#;
        let settings = Settings::new(config_str.into())?;
        let mut helper = TestHelper::new(80, 1)?;

        helper.notify(ConnectorEvents::AddOk);
        helper.draw(&settings)?;

        let mut expected = Buffer::with_lines(vec![
            "                                                                   Torrent added",
        ]);
        expected.set_style(
            Rect::new(0, 0, 80, 1),
            Style::default().fg(Color::Blue).bg(Color::Rgb(0, 0, 0)),
        );
        let buffer = helper.buffer();
        assert_eq!(buffer, &expected);
        Ok(())
    }
    #[test]
    fn notify_start_ok() -> color_eyre::Result<()> {
        let config_str = r#"
            [styles.Notification]
            info = "blue on rgb:0,0,0"
            error = "red on blue"
        "#;
        let settings = Settings::new(config_str.into())?;
        let mut helper = TestHelper::new(80, 1)?;

        helper.notify(ConnectorEvents::StartOk);

        helper.draw(&settings)?;

        let mut expected = Buffer::with_lines(vec![
            "                                                                 Torrent started",
        ]);
        expected.set_style(
            Rect::new(0, 0, 80, 1),
            Style::default().fg(Color::Blue).bg(Color::Rgb(0, 0, 0)),
        );
        let buffer = helper.buffer();
        assert_eq!(buffer, &expected);

        Ok(())
    }
    #[test]
    fn notify_pause_ok() -> color_eyre::Result<()> {
        let config_str = r#"
            [styles.Notification]
            info = "blue on rgb:0,0,0"
            error = "red on blue"
        "#;
        let settings = Settings::new(config_str.into())?;
        let mut helper = TestHelper::new(80, 1)?;

        helper.notify(ConnectorEvents::PauseOk);

        helper.draw(&settings)?;

        let mut expected = Buffer::with_lines(vec![
            "                                                                  Torrent paused",
        ]);
        expected.set_style(
            Rect::new(0, 0, 80, 1),
            Style::default().fg(Color::Blue).bg(Color::Rgb(0, 0, 0)),
        );
        let buffer = helper.buffer();
        assert_eq!(buffer, &expected);

        Ok(())
    }
    #[test]
    fn notify_forget_ok() -> color_eyre::Result<()> {
        let config_str = r#"
            [styles.Notification]
            info = "blue on rgb:0,0,0"
            error = "red on blue"
        "#;
        let settings = Settings::new(config_str.into())?;
        let mut helper = TestHelper::new(80, 1)?;

        helper.notify(ConnectorEvents::ForgetOk);

        helper.draw(&settings)?;

        let mut expected = Buffer::with_lines(vec![
            "                                          Torrent removed from list (files kept)",
        ]);
        expected.set_style(
            Rect::new(0, 0, 80, 1),
            Style::default().fg(Color::Blue).bg(Color::Rgb(0, 0, 0)),
        );
        let buffer = helper.buffer();
        assert_eq!(buffer, &expected);

        Ok(())
    }
    #[test]
    fn notify_delete_ok() -> color_eyre::Result<()> {
        let config_str = r#"
            [styles.Notification]
            info = "blue on rgb:0,0,0"
            error = "red on blue"
        "#;
        let settings = Settings::new(config_str.into())?;
        let mut helper = TestHelper::new(80, 1)?;

        helper.notify(ConnectorEvents::DeleteOk);

        helper.draw(&settings)?;

        let mut expected = Buffer::with_lines(vec![
            "                                                      Torrent deleted with files",
        ]);
        expected.set_style(
            Rect::new(0, 0, 80, 1),
            Style::default().fg(Color::Blue).bg(Color::Rgb(0, 0, 0)),
        );
        let buffer = helper.buffer();
        assert_eq!(buffer, &expected);

        Ok(())
    }
    #[test]
    fn notify_error() -> color_eyre::Result<()> {
        let config_str = r#"
            [styles.Notification]
            info = "blue on rgb:0,0,0"
            error = "red on blue"
        "#;
        let settings = Settings::new(config_str.into())?;
        let mut helper = TestHelper::new(80, 1)?;

        helper.notify(Notification::Error(String::from("Some error")));

        helper.draw(&settings)?;

        let mut expected = Buffer::with_lines(vec![
            "                                                                      Some error",
        ]);
        expected.set_style(
            Rect::new(0, 0, 80, 1),
            Style::default().fg(Color::Red).bg(Color::Blue),
        );
        let buffer = helper.buffer();
        assert_eq!(buffer, &expected);

        Ok(())
    }
    #[test]
    fn remove_notification() -> color_eyre::Result<()> {
        let config_str = r#"
            notification_timeout_millis = 0
            [styles.Notification]
            info = "blue on rgb:0,0,0"
            error = "red on blue"
        "#;
        let settings = Settings::new(config_str.into())?;
        let mut helper = TestHelper::new(80, 1)?;

        helper.notify(Notification::Error(String::from("Some error")));

        helper.draw(&settings)?;

        let mut expected = Buffer::with_lines(vec![
            "                                                                      Some error",
        ]);
        expected.set_style(
            Rect::new(0, 0, 80, 1),
            Style::default().fg(Color::Red).bg(Color::Blue),
        );
        let buffer = helper.buffer();
        assert_eq!(buffer, &expected);

        helper.component.on_user_interaction();
        helper.draw(&settings)?;

        let mut expected = Buffer::with_lines(vec![
            "                                                                                ",
        ]);
        expected.set_style(Rect::new(0, 0, 80, 1), Style::default());
        let buffer = helper.buffer();
        assert_eq!(buffer, &expected);
        Ok(())
    }
}
