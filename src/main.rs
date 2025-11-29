fn main(){
    println!("hello, world!");

    // while let Some(i) = Some(32) {
    //     println!("{}", i)
    // }
   loop {
        let some_value = Some("I am safe to unwrap");
    let none_value: Option<&str> = None;
    // println!("{:?}", none_value.unwrap());
    println!("{:?}", some_value.unwrap_or_else(||  "computed value"));
    // println!("{:?}", none_value.unwrap());


    let some_num = Some(10);
    let none_vnum: Option<i32> = None;
    // println!("{:?}", none_value.unwrap());
    println!("{:?}", some_num.unwrap_or_default());
    println!("{:?}", none_vnum.unwrap_or_default());
   }

    // for i in Some(32){
    //     println!("{}", i)
    // }


    // if let Some(i) = Some(32) {
    //     println!("{}", i)
    // }
}