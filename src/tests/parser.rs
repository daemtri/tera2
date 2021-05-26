use crate::ast::Node;
use crate::parser::Parser;

#[test]
fn can_parse_ident() {
    let tests = vec![
        "{{ hello }}",
        "{{ hello_ }}",
        "{{ hello_1 }}",
        "{{ HELLO }}",
        "{{ _1 }}",
        "{{ hey.ho }}",
        "{{ h }}",
        "{{ ho }}",
        "{{ hey.ho.hu }}",
        "{{ hey.0 }}",
        "{{ h.u }}",
        "{{ hey.ho.hu }}",
        "{{ hey.0 }}",
        "{{ h.u.x.0 }}",
        "{{ hey[0] }}",
        "{{ hey[a[0]] }}",
        "{{ hey['ho'][\"hu\"] }}",
        "{{ h['u'].x[0] }}",
    ];

    for t in tests {
        println!("{:?}", t);
        let mut parser = Parser::new(t);
        parser.parse().expect("parsed failed");
        match &parser.nodes[0] {
            Node::Expression(e) => {
                assert_eq!(e.to_string(), t.replace("{{ ", "").replace(" }}", ""))
            }
            _ => unreachable!("Got something that wasn't an expression"),
        }
    }
}

#[test]
fn can_parse_expression() {
    let tests = vec![
        // literals + basic types
        ("{{ -1 }}", "-1"),
        ("{{ 1 }}", "1"),
        ("{{ 'hello' }}", "'hello'"),
        ("{{ true }}", "true"),
        ("{{ -1.2 }}", "-1.2"),
        ("{{ 1.2 }}", "1.2"),
        ("{{ a }}", "a"),
        ("{{ -a }}", "(- a)"),
        ("{{ +a }}", "(+ a)"),
        ("{{ - a * 2 }}", "(- (* a 2))"),
        ("{{ [1, 1.2, a, 'b', true] }}", "[1, 1.2, a, 'b', true]"),
        ("{{ [1, 1.2, a, 'b', true,] }}", "[1, 1.2, a, 'b', true]"), // Allows trailing `,`
        // Actual expressions
        ("{{ 1 + 2 + 3 }}", "(+ (+ 1 2) 3)"),
        ("{{ 1 + count }}", "(+ 1 count)"),
        ("{{ 1 + 2 * 3 }}", "(+ 1 (* 2 3))"),
        ("{{ a + b * c * d + e }}", "(+ (+ a (* (* b c) d)) e)"),
        // https://github.com/pallets/jinja/issues/119
        ("{{ 2 * 4 % 8 }}", "(% (* 2 4) 8)"),
        ("{{ [1 + 1, 2, 3 * 2,] }}", "[(+ 1 1), 2, (* 3 2)]"),
        // string concat
        ("{{ hey ~ ho }}", "(~ hey ho)"),
        ("{{ 1 ~ ho }}", "(~ 1 ho)"),
        ("{{ -1.2 ~ ho }}", "(~ -1.2 ho)"),
        ("{{ [] ~ ho }}", "(~ [] ho)"),
        ("{{ 'hey' ~ ho }}", "(~ 'hey' ho)"),
        ("{{ `hello` ~ ident ~ 'ho' }}", "(~ (~ `hello` ident) 'ho')"),
        // Comparisons
        ("{{ a == b }}", "(== a b)"),
        ("{{ a != b }}", "(!= a b)"),
        ("{{ a <= b }}", "(<= a b)"),
        ("{{ a >= b }}", "(>= a b)"),
        ("{{ a < b }}", "(< a b)"),
        ("{{ a > b }}", "(> a b)"),
        ("{{ 1 + a > b }}", "(> (+ 1 a) b)"),
        ("{{ 1 + a > b * 8 }}", "(> (+ 1 a) (* b 8))"),
        // and/or
        ("{{ a and b }}", "(and a b)"),
        ("{{ a or b }}", "(or a b)"),
        (
            "{{ a + 1 == 2 or b * 3 > 10 }}",
            "(or (== (+ a 1) 2) (> (* b 3) 10))",
        ),
        // in
        ("{{ a in b }}", "(in a b)"),
        ("{{ a in b and b in c }}", "(and (in a b) (in b c))"),
        // https://github.com/mozilla/nunjucks/pull/336
        (
            "{{ msg.status in ['pending', 'confirmed'] and msg.body }}",
            "(and (in msg.status ['pending', 'confirmed']) msg.body)",
        ),
        // test
        ("{{ a is defined }}", "(is a defined)"),
        ("{{ a is not defined }}", "(not (is a defined))"),
        ("{{ a + 1 is odd }}", "(is (+ a 1) odd)"),
        ("{{ a + 1 is not odd }}", "(not (is (+ a 1) odd))"),
        ("{{ a is ending_with('s') }}", "(is a ending_with{'s'})"),
        // function calls
        (
            "{{ get_url(path=page.path, in_content=true) }}",
            "get_url{in_content=true, path=page.path}",
        ),
        ("{{ get_url() }}", "get_url{}"),
        // filters
        ("{{ a | round }}", "(| a round{})"),
        ("{{ a | round() }}", "(| a round{})"),
        ("{{ 1 + 2.1 | round }}", "(| (+ 1 2.1) round{})"),
        ("{{ [1] + [3, 2] | sort }}", "(| (+ [1] [3, 2]) sort{})"),
        ("{{ (1 + 2.1) | round }}", "(| (+ 1 2.1) round{})"),
        (
            "{{ value | json_encode | safe }}",
            "(| (| value json_encode{}) safe{})",
        ),
        (
            "{{ value | truncate(length=10) }}",
            "(| value truncate{length=10})",
        ),
        (
            "{{ get_content() | upper | safe }}",
            "(| (| get_content{} upper{}) safe{})",
        ),
        (
            "{{ admin | default or user == current_user }}",
            "(or (| admin default{}) (== user current_user))",
        ),
        (
            "{{ user == current_user or admin | default }}",
            "(or (== user current_user) (| admin default{}))",
        ),
        (
            "{{ members in interfaces | groupby(attribute='vlan') }}",
            "(in members (| interfaces groupby{attribute='vlan'}))",
        ),
        ("{{ a ~ b | upper }}", "(| (~ a b) upper{})"),
        (
            "{{ status == 'needs_restart' | ternary(truthy='restart', falsy='continue') }}",
            "(| (== status 'needs_restart') ternary{falsy='continue', truthy='restart'})",
        ),
        (
            "{{ (status == 'needs_restart') | ternary(truthy='restart', falsy='continue') }}",
            "(| (== status 'needs_restart') ternary{falsy='continue', truthy='restart'})",
        ),
        // Macro calls
        (
            "{{ macros::input(label='Name', type='text') }}",
            "macros::input{label='Name', type='text'}",
        ),
        ("{{ macros::input() | safe }}", "(| macros::input{} safe{})"),
        // Parentheses
        ("{{ ((1)) }}", "1"),
        ("{{ (2 * 3) / 10 }}", "(/ (* 2 3) 10)"),
        ("{{ (2 * 3) / 10 }}", "(/ (* 2 3) 10)"),
        // not
        ("{{ not a }}", "(not a)"),
        ("{{ not b * 1 }}", "(not (* b 1))"),
        ("{{ not a and 1 + b > 3 }}", "(and (not a) (> (+ 1 b) 3))"),
        (
            "{{ not id and not true and not 1 + c }}",
            "(and (and (not id) (not true)) (not (+ 1 c)))",
        ),
        ("{{ a not in b }}", "(not (in a b))"),
        (
            "{{ a is defined and b is not defined(1, 2) }}",
            "(and (is a defined) (not (is b defined{1, 2})))",
        ),
        (
            "{{ a is defined and not b is defined(1, 2) }}",
            "(and (is a defined) (not (is b defined{1, 2})))",
        ),
        (
            "{{ not admin | default(val=true) }}",
            "(not (| admin default{val=true}))",
        ),
    ];

    for (t, expected) in tests {
        println!("{:?}", t);
        let mut parser = Parser::new(t);
        parser.parse().expect("parsed failed");
        match &parser.nodes[0] {
            Node::Expression(e) => {
                assert_eq!(e.to_string(), expected)
            }
            _ => unreachable!("Got something that wasn't an expression"),
        }
    }
}

// TODO
// #[test]
// fn can_parse_expression_constant_folding() {
//     // TODO
//
//     let tests = vec![
//         // TODO
//         // https://github.com/Keats/tera/blob/master/src/parser/tests/parser.rs#L1074
//         // ("`hello` ~ 'hey'", "'hellohey'"),
//         // ("1 ~ 'ho'", "'1ho'"),
//         // comparisons
//         // ("1 == 1", "true"),
//         // ("1 == '1'", "false"),
//         // ("1 == 0", "false"),
//     ];
//
//     for (input, expected) in tests {
//         let mut parser = Parser::new(input);
//         assert_eq!(parser.parse_expression(0).to_string(), expected);
//     }
// }