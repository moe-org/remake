use crate::errors::ParseError;
use crate::format::definition::Platform;
use crate::format::{Target, CommandsRunable, Command};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use ahash::AHashMap;

pub struct ParseOption {}

pub struct ParsedRemake {
    pub targets: AHashMap<Arc<String>, Arc<Target>>,
}

struct ByteReader<'a> {
    source: &'a [u8],
    length: usize,
    index: usize,
}

impl<'a> ByteReader<'a> {
    pub fn new(byte_array: &'a [u8]) -> ByteReader<'a> {
        return ByteReader {
            source: byte_array,
            length: byte_array.len(),
            index: 0,
        };
    }

    pub fn read(&mut self, size: usize) -> Option<&'a [u8]> {
        let end = self.index + size;

        if end >= self.length {
            return None;
        }

        let buf = &self.source[self.index..(self.index + size)];

        self.index += size;

        return Some(buf);
    }

    fn read_some<T, F>(&mut self, size: usize, func: F) -> Option<T>
    where
        F: Fn(&'a [u8]) -> T,
    {
        let result = self.read(size);

        if result.is_none() {
            return None;
        } else {
            return Some(func(result.unwrap()));
        }
    }

    pub fn read_u64(&mut self) -> Option<u64> {
        return self.read_some(8, |read_bytes: &[u8]| -> u64 {
            u64::from_le_bytes(read_bytes.try_into().unwrap())
        });
    }

    pub fn read_boolean(&mut self) -> Option<bool> {
        return self.read_some(1, |read_bytes: &[u8]| -> bool {
            *read_bytes.get(0).unwrap() != 0u8
        });
    }

    pub fn read_string(&mut self) -> Option<&'a str> {
        let length = self.read_u64();

        if length.is_none() {
            return None;
        }

        return self.read_some(
            usize::try_from(length.unwrap()).unwrap(),
            |read_bytes: &'a [u8]| -> &'a str { std::str::from_utf8(read_bytes).unwrap() },
        );
    }

    pub fn get_span(&self) -> (u64, u64) {
        return ((self.index - 1) as u64, self.index as u64);
    }

    pub fn is_end(&self) -> bool{
        self.index == self.length
    }

    pub fn read_string_array(&mut self) -> Option<Arc<Vec<String>>>{
        let length = self.read_u64();

        if length.is_none() {
            return None;
        }

        let length =length.unwrap();
        
        let mut vecs = Vec::<String>::with_capacity(usize::try_from(length).unwrap());

        for _ in 0..length{
            let read = self.read_string();

            if read.is_none(){
                return None
            }

            vecs.push(
                String::from(read.unwrap())
            )
        }

        return Some(Arc::new(vecs));
    }

    pub fn read_string_map(&mut self) -> Option<AHashMap<String,String>>{
        let length = self.read_u64();

        if length.is_none() {
            return None;
        }

        let length =length.unwrap();

        let mut map = AHashMap::<String,String>::with_capacity(usize::try_from(length).unwrap());

        for _ in 0..length{
            let k = self.read_string();
            let v = self.read_string();

            if k.is_none() || v.is_none(){
                return None;
            }

            map.insert(String::from(k.unwrap()), String::from(v.unwrap()));
        }
        return Some(map);
    }

}

/// read a command
fn parse_command<'a>(reader:&mut ByteReader<'a>) -> Option<CommandsRunable>{
    // read program name
    let name = reader.read_string();
    let args = reader.read_string_array();
    let ignore_errors = reader.read_boolean();
    let envs = reader.read_string_map();
    let cwd = reader.read_string();

    if name.is_none() || args.is_none() || ignore_errors.is_none() || envs.is_none() || cwd.is_none(){
        return None;
    }
    
    return Some(
        CommandsRunable{
            command: Arc::new(Command{
            executable: Arc::new(String::from(name.unwrap())),
            arguments: args.unwrap().to_vec(),
            ignore_error:AtomicBool::new(ignore_errors.unwrap()),
            environments:std::sync::RwLock::new(envs.unwrap()),
            work_dir:Arc::new(String::from(cwd.unwrap()))
            })
        }
    )
}

/// read a target
fn parse_target<'a>(reader:&mut ByteReader<'a>) -> Option<Arc<Target>>{

    // reader target name
    let name = reader.read_string();

    if name.is_none(){
        return None;
    }
    let name = name.unwrap();

    // read the dependences
    let dependences = reader.read_string_array();

    if dependences.is_none(){
        return None;
    }
    let dependences = dependences.unwrap();

    // read commands
    let command_count = reader.read_u64();
    if command_count.is_none(){
        return None;
    }
    let command_count = command_count.unwrap();

    let mut commands = Vec::<CommandsRunable>::with_capacity(usize::try_from(command_count).unwrap());

    for _ in 0..command_count{
        let cmd = parse_command(reader);

        if cmd.is_none(){
            return None;
        }
        else{
            commands.push(cmd.unwrap());
        }
    }

    let commands = commands;

    return Some(Arc::new(Target{
        name:Arc::new(String::from(name)),
        dependences:dependences,
        commands:Arc::new(commands)
    }));
}


fn parse_prefix<'a>(reader:&mut ByteReader<'a>) -> Option<ParseError>{
    let taken = reader.read(6);

    if taken.is_none() {
        return Some(ParseError::from_exceptional_eof());
    }

    let taken = taken.unwrap();

    if *taken.get(0).unwrap() != b'r'
        || *taken.get(1).unwrap() != b'e'
        || *taken.get(2).unwrap() != b'm'
        || *taken.get(3).unwrap() != b'a'
        || *taken.get(4).unwrap() != b'k'
        || *taken.get(5).unwrap() != b'e'
    {
        return Some(ParseError {
            source: None,
            source_span: Some((0, 6)),
            reason: Some(String::from("the bytes do not start with `remake`")),
        });
    }   

    return None;
}

fn parse_platform<'a>(reader:&mut ByteReader<'a>) -> Option<ParseError>{
    let taken = reader.read_u64();

    if taken.is_none() {
        return Some(ParseError::from_exceptional_eof());
    }

    let p = crate::format::definition::Platform::try_from(taken.unwrap());

    if p.is_err(){
        return Some(ParseError { source: None, source_span: Some(reader.get_span()), reason: 
        Some(String::from("the platform number is unknown. format may be broken.")) });
    }

    return match p.unwrap() {
        Platform::Unix => {
            cfg_if::cfg_if! {
            if #[cfg(unix)] {
                None
            }
            else{
                Some(
                    ParseError {
                        source: None,
                        source_span: Some(reader.get_span()),
                        reason:  Some(String::from("the platform is not right"))
                    }
                )
            }
            }
        }
        Platform::Freebsd => {
            panic!("no bsd from remake");
        }
        Platform::Mac => {
            cfg_if::cfg_if! {
                if #[cfg(target_os = "macos")] {
                    None
                }
                else{
                     Some(
                        ParseError {
                            source: None,
                            source_span: Some(reader.get_span()),
                            reason:  Some(String::from("the platform is not right"))
                        }
                    )
                }
            }
        }
        Platform::Window => {
            cfg_if::cfg_if! {
                if #[cfg(windows)] {
                    None
                }
                else{
                    Some(
                        ParseError {
                            source: None,
                            source_span: Some(reader.get_span()),
                            reason:  Some(String::from("the platform is not right"))
                        }
                    )
                }
            }
        }
    }
}

fn parse_version<'a>(reader:&mut ByteReader<'a>) -> Option<ParseError>{
    let version = reader.read_u64();

    if version.is_none(){
        return Some(ParseError::from_exceptional_eof());
    }
    else{
        if env!("CARGO_PKG_VERSION_MAJOR").parse::<u64>().unwrap() != version.unwrap(){
            return Some(ParseError { source: None, 
                source_span: Some(reader.get_span()), 
                reason: Some(String::from("the version is not match")) })
        }
    }
    return None;
}

/// parse bytes
pub fn parse_from_bytes(
    bytes_vec: Vec<u8>
) -> Result<ParsedRemake, ParseError> {
    let bytes = bytes_vec.into_boxed_slice();
    let bytes_ref = &*bytes;
    let mut reader = ByteReader::new(bytes_ref);

    // 检查remake前缀
    let prefix = parse_prefix(&mut reader);

    if prefix.is_some(){
        return Err(prefix.unwrap());
    }

    // 检查平台
    let platform = parse_platform(&mut reader);

    if platform.is_some(){
        return Err(platform.unwrap());
    }

    // 解析version
    let version = parse_version(&mut reader);

    if version.is_some(){
        return Err(version.unwrap());
    }

    // 获取targets数量
    let target_count = reader.read_u64();

    if target_count.is_none(){
        return Err(ParseError::from_exceptional_eof());
    }

    let target_count = target_count.unwrap();

    let mut targets : AHashMap<Arc<String>, Arc<Target>> = AHashMap::with_capacity(
        usize::try_from(target_count).unwrap()
    );

    // 读取
    for _ in 0..target_count {
        let target = parse_target(&mut reader);

        if target.is_none(){
            return Err(ParseError::from_exceptional_eof());
        }
        else{
            let target = target.unwrap();
            targets.insert(
                target.name.clone(),
                target
            );
        }
    }

    // Check it is end
    if !reader.is_end(){
        return Err(
            ParseError{
                source:None,
                source_span:Some(reader.get_span()),
                reason: Some(String::from("All content has read but there are some bytes left"))
            }
        )
    }

    Ok(
        ParsedRemake { targets }
    )
}
