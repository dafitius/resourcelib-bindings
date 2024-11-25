use resourcelib_ffi::{ResourceConverter, ResourceLib, WoaVersion};

fn main(){
  println!("Hello world");

  println!("HM2016 types:");
  if let Ok(types) = ResourceLib::supported_resource_types(WoaVersion::HM2016){
    for t in types{
      println!("{:?}",t);
    }
  }

  println!("HM2 types:");
  if let Ok(types) = ResourceLib::supported_resource_types(WoaVersion::HM2){
    for t in types{
      println!("{:?}",t);
    }
  }

  println!("HM3 types:");
  if let Ok(types) = ResourceLib::supported_resource_types(WoaVersion::HM3){
    for t in types{
      println!("{:?}",t);
    }
  }

  println!("--------------------------------------------------------");

  let rc2016 = ResourceConverter::new(WoaVersion::HM2016,"TEMP");
  let rc3 = ResourceConverter::new(WoaVersion::HM3,"TEMP");

  match rc3 {
    Ok(rc) => {
      let json = rc.resource_file_to_json_string("D:\\David\\Hitman-modding\\Tools\\rpkgTools\\2.19\\chunk0\\TEMP\\0057C0E23328F131.TEMP");
      println!("{:?}", json);
    },
    Err(e) => {
      println!("{}",e);
    }
  }

  println!("--------------------------------------------------------");

  match rc2016 {
    Ok(rc) => {
      let json = rc.resource_file_to_json_string("D:\\David\\Hitman-modding\\Tools\\rpkgTools\\2.19\\chunk0\\TEMP\\008F5EC55CA1BBE5.TEMP");
      println!("{:?}", json);
    },
    Err(e) => {
      println!("{}",e);
    }
  }

  println!("------------------------------------------------------------");

}