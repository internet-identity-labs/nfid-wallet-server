pub enum ErrorMessage {
    NoSuchApp,
}

impl ErrorMessage {
    pub fn value(&self) -> String {
        match *self {
            ErrorMessage::NoSuchApp => "No such application.".to_string(),
        }
    }
}

pub enum Application {
    Metamask,
}

impl Application {
    pub fn value(&self) -> String {
        match *self {
            Application::Metamask => "METAMASK".to_string(),
        }
    }
}

pub enum Message {
    Message,
}

impl Message {
    pub fn value(&self) -> String {
        match *self {
            Message::Message => "Hi there from NFID! Sign this message to prove you own this wallet and we’ll log you in. This won’t cost you any Ether.".to_string(),
        }
    }
}