use crate::object::*;
impl TryFrom<DirEntry> for FsObject{
    type Error = ();
    fn try_from(dir_entry: DirEntry) -> Result<Self, Self::Error> {
        let object_type = match ObjectType::try_from(dir_entry.path().as_ref()){
            Ok(object_type) => {object_type},
            Err(_) => {return Err(())}
        };
        let metadata = match dir_entry.metadata(){
            Ok(data) => {data},
            Err(_) => {return Err(())}
        };
        let path: PathBuf = dir_entry.path();
        let full_file_name = match path.file_name(){
            Some(full_file_name) => {full_file_name},
            None => {return Err(());}
        };
        let full_file_name_str = match full_file_name.to_str(){
            Some(full_file_name_str) => {full_file_name_str},
            None => return Err(())
        };
        let name: NameString = match NameString::from_full_file_name(full_file_name_str){
            Ok(name) => {name},
            Err(_char) => return Err(())
        };
        return Ok(FsObject{
            object_type,
            name,
            metadata,
            path
        })
    }
}
impl TryFrom<&Path> for ObjectType {
    type Error = ();
    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let file_extension = match path.extension(){
            Some(extension_os_str) => {extension_os_str.to_string_lossy()},
            None => {return Err(());}
        };
        match &*file_extension{
            "regular" => {return Ok(ObjectType::RegularChat)},
            "textscript" => {return Ok(ObjectType::TextScript)}
            _ => {return Err(());}
        }
    }
}