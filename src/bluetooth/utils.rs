pub trait U8ArrayExtension {
    fn format_as_hex(&self) -> String;
    fn format_as_hex_with_lenght(&self, lenght: usize) -> String;
}

impl U8ArrayExtension for [u8] {
    fn format_as_hex(&self) -> String {
        let mut s = String::with_capacity(self.len() * 3);
        for n in self {
            s.push_str(format!("{:02x?} ", n).as_str());
        }
        s.pop();
        s
    }
    
    fn format_as_hex_with_lenght(&self, lenght: usize) -> String {
        self[0..lenght].format_as_hex()
    }    
}
