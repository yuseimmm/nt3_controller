use std::io::Write;
use zenoh::{bytes::ZBytes, Session, Wait};

pub struct Driver {
    session: Session,
    key_expr: String,
}

impl Driver {
    pub fn new(session: Session, key_expr: String) -> Driver {
        Self { session, key_expr }
    }

    pub fn send_code(&self, id: u8, angle: u8) {
        let buf: Vec<u8> = vec![id, angle];
        let mut writer = ZBytes::writer();
        writer.write(&buf).unwrap();

        let zbytes = writer.finish();
        self.session.put(&self.key_expr, zbytes).wait().unwrap();
    }
}
