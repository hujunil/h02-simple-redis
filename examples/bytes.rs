use bytes::{Buf, BufMut, BytesMut};

fn main() {
    t1();
    t2();
    t3();
    t4();
    t5();
    t6();
}

fn t1() {
    let mut buf = BytesMut::with_capacity(1024);
    assert_eq!(buf.len(), 0);
    buf.put(&b"hello world"[..]);
    buf.put_u16(1234);
    println!("before buf:{:?}", buf);
    let a = buf.split();
    println!("after buf:{:?}", buf);
    println!("a:{:?}", a);
}

fn t2() {
    let mut bytes = BytesMut::with_capacity(64);
    assert!(bytes.is_empty());
    println!("bytes capacity:{:?}", bytes.capacity());
    bytes.put(b"hello world".as_ref());
    assert_eq!(&bytes[..], b"hello world");
}

fn t3() {
    let mut buf = BytesMut::with_capacity(1024);
    buf.put(&b"hello world"[..]);

    let other = buf.split();

    assert!(buf.is_empty());
    assert_eq!(1013, buf.capacity());

    assert_eq!(other, b"hello world"[..]);
}

fn t4() {
    let buf = BytesMut::from(&b"hello"[..]);
    println!("buf len:{:?}", buf.len());
    println!("buf capacity:{:?}", buf.capacity());
}

fn t5() {
    let mut buf = b"0123456789".as_slice();
    assert_eq!(buf.chunk(), b"0123456789");
    buf.advance(5);
    assert_eq!(buf.chunk(), b"56789");
    buf.advance(2);
    assert_eq!(buf.chunk(), b"789");
}

fn t6() {
    let mut buf = BytesMut::from(b"0123456789".as_slice());
    assert_eq!(buf.chunk(), b"0123456789");
    buf.advance(5);
    let s1 = buf.split_to(2);
    assert_eq!(s1, b"56"[..]);
    assert_eq!(buf.chunk(), b"789");
}
