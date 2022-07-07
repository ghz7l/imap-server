use crate::{
    core::{receiver::Request, utf7::utf7_maybe_decode},
    protocol::{rename, ProtocolVersion},
};

impl Request {
    pub fn parse_rename(self, version: ProtocolVersion) -> crate::core::Result<rename::Arguments> {
        match self.tokens.len() {
            2 => {
                let mut tokens = self.tokens.into_iter();
                Ok(rename::Arguments {
                    mailbox_name: utf7_maybe_decode(
                        tokens
                            .next()
                            .unwrap()
                            .unwrap_string()
                            .map_err(|v| (self.tag.as_ref(), v))?,
                        version,
                    ),
                    new_mailbox_name: utf7_maybe_decode(
                        tokens
                            .next()
                            .unwrap()
                            .unwrap_string()
                            .map_err(|v| (self.tag.as_ref(), v))?,
                        version,
                    ),
                    tag: self.tag,
                })
            }
            0 => Err(self.into_error("Missing argument.")),
            1 => Err(self.into_error("Missing new mailbox name.")),
            _ => Err(self.into_error("Too many arguments.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::receiver::Receiver,
        protocol::{rename, ProtocolVersion},
    };

    #[test]
    fn parse_rename() {
        let mut receiver = Receiver::new();

        for (command, arguments) in [
            (
                "A142 RENAME \"my funky mailbox\" Private\r\n",
                rename::Arguments {
                    mailbox_name: "my funky mailbox".to_string(),
                    new_mailbox_name: "Private".to_string(),
                    tag: "A142".to_string(),
                },
            ),
            (
                "A142 RENAME {1+}\r\na {1+}\r\nb\r\n",
                rename::Arguments {
                    mailbox_name: "a".to_string(),
                    new_mailbox_name: "b".to_string(),
                    tag: "A142".to_string(),
                },
            ),
        ] {
            assert_eq!(
                receiver
                    .parse(&mut command.as_bytes().iter())
                    .unwrap()
                    .parse_rename(ProtocolVersion::Rev2)
                    .unwrap(),
                arguments
            );
        }
    }
}
