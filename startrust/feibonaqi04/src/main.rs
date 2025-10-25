use std::io;
fn main() {
    println!("Hello, world!");
    let mut num = String::new();
    io::stdin().read_line(&mut num).expect("error");
    // io::stdin().read_line(&mut num).expect();
    let mut num:i32=match num.trim().parse(){
        Ok(num)=>num,
        // Err(_)=>println!("error"),
        Err(_)=>0,
    };

    let mut num1=0;
    let mut num2=1;

    let mut num3;
    if num==1 {
        println!("{}",num1);
    }
    else if num==2 {
        println!("{}",num1);
        println!("{}",num2);
    }
    else if num>=3 {
        println!("{}",num1);
        println!("{}",num2);
        while num>2 {
            num=num-1;
            num3=num1+num2;
            num1=num2;
            num2=num3;
            println!("{}",num3);
        }
    }
    
}
