iface foo<T> { }

fn bar(x: foo<uint>) -> foo<int> {
    ret (x as foo::<int>);
    //~^ ERROR mismatched types: expected `foo<int>` but found `foo<uint>`
}

fn main() {}
