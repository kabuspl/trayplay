use std::process::{Command, Stdio};

pub enum ClickedButton {
    Ok,
    Yes,
    No,
    Cancel,
    None,
}

#[derive(PartialEq)]
#[allow(dead_code)]
pub enum MessageBoxButtons {
    Ok,
    YesNo,
    YesNoCancel,
}

pub struct MessageBox {
    buttons: MessageBoxButtons,
    label: String,
    title: Option<String>,
}

#[allow(dead_code)]
impl MessageBox {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            buttons: MessageBoxButtons::Ok,
            title: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn buttons(mut self, buttons: MessageBoxButtons) -> Self {
        self.buttons = buttons;
        self
    }

    pub fn show(&self) -> Result<ClickedButton, std::io::Error> {
        let mut command = Command::new("kdialog");

        if let Some(title) = &self.title {
            command.args(["--title", title]);
        }

        match self.buttons {
            MessageBoxButtons::Ok => {
                command.arg("--msgbox");
            }
            MessageBoxButtons::YesNo => {
                command.arg("--yesno");
            }
            MessageBoxButtons::YesNoCancel => {
                command.arg("--yesnocancel");
            }
        }

        command.arg(&self.label);

        let mut child = command.spawn()?;
        Ok(match child.wait()?.code() {
            Some(code) => match code {
                0 => {
                    if self.buttons == MessageBoxButtons::Ok {
                        ClickedButton::Ok
                    } else {
                        ClickedButton::Yes
                    }
                }
                1 => ClickedButton::No,
                2 => ClickedButton::Cancel,
                _ => ClickedButton::None,
            },
            None => ClickedButton::None,
        })
    }
}

#[allow(dead_code)]
pub enum InputBoxType {
    Text,
    Password,
    NewPassword,
    TextArea,
}

pub struct InputBox {
    label: String,
    title: Option<String>,
    initial: String,
    input_box_type: InputBoxType,
}

impl InputBox {
    pub fn new(label: impl Into<String>, input_box_type: InputBoxType) -> Self {
        Self {
            label: label.into(),
            title: None,
            initial: "".into(),
            input_box_type,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn initial(mut self, initial: impl Into<String>) -> Self {
        self.initial = initial.into();
        self
    }

    pub fn show(&self) -> Result<Option<String>, std::io::Error> {
        let mut command = Command::new("kdialog");

        if let Some(title) = &self.title {
            command.args(["--title", title]);
        }

        command.arg(match self.input_box_type {
            InputBoxType::Text => "--inputbox",
            InputBoxType::Password => "--password",
            InputBoxType::NewPassword => "--newpassword",
            InputBoxType::TextArea => "--textinputbox",
        });

        command.arg(&self.label);

        let child = command.stdout(Stdio::piped()).spawn()?;

        let output = child.wait_with_output()?;

        Ok(if output.status.success() {
            Some(String::from_utf8(output.stdout).unwrap())
        } else {
            None
        })
    }
}
