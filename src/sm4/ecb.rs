use crate::sm4::core::Crypto;
use crate::sm4::Cryptographer;

/// ECB: Electronic Code Book
///
/// 电子密码本模式
///
/// ### 不应该使用
///
/// 优点：
/// * 简单
/// * 快速
/// * 支持并行计算(加密、解密）
///
/// 缺点：
/// * 明文中的重复排列会反映在密文中
/// * 通过删除、替换密文分组可以对明文进行操作
/// * 对包含某些比特错误的密文进行解密时，对应的分组会出错
/// * 不能抵御重放攻击
pub struct CryptoMode {
    crypto: Crypto,
}

impl CryptoMode {
    pub fn new(key: &[u8]) -> Self {
        crate::sm4::ecb::CryptoMode { crypto: Crypto::init(key) }
    }
}


impl Cryptographer for CryptoMode {
    fn encrypt_bytes(&self, plain: &[u8]) -> Vec<u8> {
        // 计算分组，每个分组应该是满16字节。最后一个分组要么是明文+填充总共满足16字节，要么是全填充16字节
        // 填充数据原则：(16-remainder)个(16-remainder)
        let (quotients, remainder) = (plain.len() / 16, plain.len() % 16);
        let mut out: Vec<u8> = Vec::new();
        // 对分组分别进行加密形成分组密文
        for i in 0..quotients {
            let cipher = self.crypto.encrypt(&plain[i * 16..(i + 1) * 16]);
            out.extend_from_slice(&cipher);
        }

        if remainder != 0 {
            // 如果数据长度除以16有余数，那就补充(16-余数)个(16-余数)
            let mut last = [(16 - remainder) as u8; 16];
            last[..remainder].copy_from_slice(&plain[quotients * 16..]);
            let cipher = self.crypto.encrypt(&last);
            out.extend_from_slice(&cipher);
        } else {
            // 如果数据长度正好是16的倍数，那就补充16个字节,补充数据为0x10=16
            let cipher = self.crypto.encrypt(&[0x10; 16]);
            out.extend_from_slice(&cipher);
        }
        out
    }

    fn decrypt_bytes(&self, cipher: &[u8]) -> Vec<u8> {
        let (quotients, remainder) = (cipher.len() / 16, cipher.len() % 16);
        if remainder != 0 {
            panic!("The cipher‘s length must be a multiple of 16 bytes.");
        }

        let mut out: Vec<u8> = Vec::new();
        for i in 0..quotients {
            let block = self.crypto.decrypt(&cipher[i * 16..(i + 1) * 16]);
            block.iter().for_each(|e| out.push(*e));
        }

        let last_byte = out[cipher.len() - 1];
        out.resize(cipher.len() - last_byte as usize, 0);
        out
    }
}


#[cfg(test)]
mod tests {
    use crate::sm4::Cryptographer;
    use crate::sm4::ecb::CryptoMode;

    #[test]
    fn main() {
        let key = hex::decode("0123456789abcdeffedcba9876543210").unwrap();
        let plain = "Hello World, 哈喽，世界";

        let c = CryptoMode::new(&key);
        let cipher = c.encrypt(String::from(plain));
        let text = c.decrypt(cipher);

        assert_eq!(plain, text);
    }
}
