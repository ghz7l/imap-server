use crate::{
    core::{receiver::Request, utf7::utf7_maybe_decode},
    protocol::{subscribe, ProtocolVersion},
};

impl Request {
    pub fn parse_subscribe(
        self,
        version: ProtocolVersion,
    ) -> crate::core::Result<subscribe::Arguments> {
        match self.tokens.len() {
            1 => Ok(subscribe::Arguments {
                mailbox_name: utf7_maybe_decode(
                    self.tokens
                        .into_iter()
                        .next()
                        .unwrap()
                        .unwrap_string()
                        .map_err(|v| (self.tag.as_ref(), v))?,
                    version,
                ),
                tag: self.tag,
            }),
            0 => Err(self.into_error("Missing mailbox name.")),
            _ => Err(self.into_error("Too many arguments.")),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::receiver::Receiver,
        protocol::{subscribe, ProtocolVersion},
    };

    #[test]
    fn parse_subscribe() {
        let mut receiver = Receiver::new();

        for (command, arguments) in [
            (
                "A142 SUBSCRIBE #news.comp.mail.mime\r\n",
                subscribe::Arguments {
                    mailbox_name: "#news.comp.mail.mime".to_string(),
                    tag: "A142".to_string(),
                },
            ),
            (
                "A142 SUBSCRIBE \"#news.comp.mail.mime\"\r\n",
                subscribe::Arguments {
                    mailbox_name: "#news.comp.mail.mime".to_string(),
                    tag: "A142".to_string(),
                },
            ),
        ] {
            assert_eq!(
                receiver
                    .parse(&mut command.as_bytes().iter())
                    .unwrap()
                    .parse_subscribe(ProtocolVersion::Rev2)
                    .unwrap(),
                arguments
            );
        }
    }
}
