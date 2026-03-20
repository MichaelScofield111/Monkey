## pratt parser(递归 + 迭代)

1 + (2 + 3 + 4) + 2 + 3

1. precedence

A + B \* C
parse(precedence, remain) = parse("+", 'B \* C')
parse(precedence, remain) {
// "+": 3
// for remain not end {
op = next_token()
parse(op, remain[1:])
}
}

## function

fn <Params list> <Body>
<Params List> = (Indentifier1, Indentifier2...)
let f = fn(a, b) {return a + b}
