use crate::ast::expression::Expression;
use crate::ast::identifier::Identifier;
use crate::ast::node::Node;
use crate::ast::types::{type_with_identifier_parser, Type};
use crate::choose_result;
use crate::parser::{Parse, Parser};
use crate::parser_error::*;
use papyrus_compiler_lexer::syntax::keyword_kind::KeywordKind;
use papyrus_compiler_lexer::syntax::operator_kind::OperatorKind;
use papyrus_compiler_lexer::syntax::token::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'source> {
    /// 'int x = 1'
    VariableDefinition {
        type_node: Node<Type<'source>>,
        name: Node<Identifier<'source>>,
        initial_value: Option<Node<Expression<'source>>>,
    },
    /// 'Return x'
    Return(Option<Node<Expression<'source>>>),
    /// 'x = y'
    Assignment {
        lhs: Node<Expression<'source>>,
        kind: Node<AssignmentKind>,
        rhs: Node<Expression<'source>>,
    },
    If {
        if_path: Node<ConditionalPath<'source>>,
        other_paths: Option<Vec<Node<ConditionalPath<'source>>>>,
        else_path: Option<Vec<Node<Statement<'source>>>>,
    },
    While(Node<ConditionalPath<'source>>),
    Expression(Node<Expression<'source>>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConditionalPath<'source> {
    pub condition: Node<Expression<'source>>,
    pub statements: Option<Vec<Node<Statement<'source>>>>,
}

impl<'source> ConditionalPath<'source> {
    pub fn new(
        condition: Node<Expression<'source>>,
        statements: Option<Vec<Node<Statement<'source>>>>,
    ) -> Self {
        Self {
            condition,
            statements,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AssignmentKind {
    /// '='
    Normal,
    /// '+='
    Addition,
    /// '-='
    Subtraction,
    /// '*='
    Multiplication,
    /// '/='
    Division,
    /// '%='
    Modulus,
}

impl<'source> Parse<'source> for AssignmentKind {
    fn parse(parser: &mut Parser<'source>) -> ParserResult<'source, Self> {
        choose_result!(
            parser.optional_result(|parser| parser
                .expect_operator(OperatorKind::Assignment)
                .map(|_| AssignmentKind::Normal)),
            parser.optional_result(|parser| parser
                .expect_operator(OperatorKind::AdditionAssignment)
                .map(|_| AssignmentKind::Addition)),
            parser.optional_result(|parser| parser
                .expect_operator(OperatorKind::SubtractionAssignment)
                .map(|_| AssignmentKind::Subtraction)),
            parser.optional_result(|parser| parser
                .expect_operator(OperatorKind::MultiplicationAssignment)
                .map(|_| AssignmentKind::Multiplication)),
            parser.optional_result(|parser| parser
                .expect_operator(OperatorKind::DivisionAssignment)
                .map(|_| AssignmentKind::Division)),
            parser.optional_result(|parser| parser
                .expect_operator(OperatorKind::ModulusAssignment)
                .map(|_| AssignmentKind::Modulus)),
        )
    }
}

impl<'source> Parse<'source> for Statement<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParserResult<'source, Self> {
        choose_result!(
            parser.optional_result(parse_return_statement),
            parser.optional_result(parse_if_statement),
            parser.optional_result(parse_while_statement),
            parser.optional_result(parse_define_statement),
            parser.optional_result(parse_assign_statement),
            parser.optional_result(parse_expression_statement)
        )
    }
}

/// ```ebnf
///
/// <assign statement> ::= (<l-value> '=' <expression>) |
///                        (<l-value> '+=' <expression>) |
///                        (<l-value> '-=' <expression>) |
///                        (<l-value> '*=' <expression>) |
///                        (<l-value> '/=' <expression>) |
///                        (<l-value> '%=' <expression>)
/// <l-value>          ::= ([<expression> '.'] <identifier>) |
///                        (<expression> '[' <expression> ']')
///
/// ```
fn parse_assign_statement<'source>(
    parser: &mut Parser<'source>,
) -> ParserResult<'source, Statement<'source>> {
    let l_value = parser.parse_node::<Expression>()?;
    let assignment_kind = parser.parse_node::<AssignmentKind>()?;
    let r_value = parser.parse_node::<Expression>()?;

    Ok(Statement::Assignment {
        lhs: l_value,
        kind: assignment_kind,
        rhs: r_value,
    })
}

fn parse_expression_statement<'source>(
    parser: &mut Parser<'source>,
) -> ParserResult<'source, Statement<'source>> {
    let expression = parser.parse_node::<Expression>()?;
    Ok(Statement::Expression(expression))
}

/// ```ebnf
/// 'Return' [<expression>]
/// ```
fn parse_return_statement<'source>(
    parser: &mut Parser<'source>,
) -> ParserResult<'source, Statement<'source>> {
    parser.expect_keyword(KeywordKind::Return)?;
    let expression = parser.parse_node_optional::<Expression>();

    Ok(Statement::Return(expression))
}

/// ```ebnf
/// <define statement> ::= <type> <identifier> ['=' <expression>]
/// ```
fn parse_define_statement<'source>(
    parser: &mut Parser<'source>,
) -> ParserResult<'source, Statement<'source>> {
    let (type_node, name) = type_with_identifier_parser(parser)?;
    let initial_value = parser.optional(|parser| {
        parser.expect_operator(OperatorKind::Assignment)?;
        parser.parse_node::<Expression>()
    });

    Ok(Statement::VariableDefinition {
        type_node,
        name,
        initial_value,
    })
}

/// ```ebnf
/// <conditional path> ::= <expression> <statement>*
/// ```
fn parse_conditional_path<'source>(
    parser: &mut Parser<'source>,
) -> ParserResult<'source, ConditionalPath<'source>> {
    let condition = parser.parse_node::<Expression>()?;
    let mut statements = Vec::<Node<Statement>>::new();

    loop {
        let next_token = parser.peek_token();

        match next_token {
            Some(Token::Keyword(KeywordKind::EndIf)) => break,
            Some(Token::Keyword(KeywordKind::EndWhile)) => break,
            Some(Token::Keyword(KeywordKind::ElseIf)) => break,
            Some(Token::Keyword(KeywordKind::Else)) => break,
            _ => statements.push(parser.parse_node::<Statement>()?),
        }
    }

    let statements = {
        if statements.is_empty() {
            None
        } else {
            Some(statements)
        }
    };

    Ok(ConditionalPath {
        condition,
        statements,
    })
}

/// ```ebnf
/// <if statement> ::= 'If' <conditional path>
///                    ['ElseIf' <conditional path>]*
///                    ['Else' <conditional path>]
///                    'EndIf'
/// ```
fn parse_if_statement<'source>(
    parser: &mut Parser<'source>,
) -> ParserResult<'source, Statement<'source>> {
    parser.expect_keyword(KeywordKind::If)?;

    let if_path = parser.with_node(parse_conditional_path)?;

    let mut other_paths = Vec::<Node<ConditionalPath>>::new();
    let mut else_path: Option<Vec<Node<Statement>>> = None;

    while parser.peek_token() != Some(&Token::Keyword(KeywordKind::EndIf)) {
        let next_token = parser.peek_token();
        match next_token {
            Some(Token::Keyword(KeywordKind::ElseIf)) => {
                parser.expect_keyword(KeywordKind::ElseIf)?;
                other_paths.push(parser.with_node(parse_conditional_path)?);
            }
            Some(Token::Keyword(KeywordKind::Else)) => {
                parser.expect_keyword(KeywordKind::Else)?;
                else_path = parser.optional_parse_node_until_keyword(KeywordKind::EndIf)?;
                break;
            }
            _ => break,
        }
    }

    parser.expect_keyword(KeywordKind::EndIf)?;

    let other_paths = {
        if other_paths.is_empty() {
            None
        } else {
            Some(other_paths)
        }
    };

    Ok(Statement::If {
        if_path,
        other_paths,
        else_path,
    })
}

/// ```ebnf
/// 'While' <conditional path>
/// 'EndWhile'
/// ```
fn parse_while_statement<'source>(
    parser: &mut Parser<'source>,
) -> ParserResult<'source, Statement<'source>> {
    parser.expect_keyword(KeywordKind::While)?;
    let conditional_path = parser.with_node(parse_conditional_path)?;
    parser.expect_keyword(KeywordKind::EndWhile)?;

    Ok(Statement::While(conditional_path))
}

#[cfg(test)]
mod test {
    use crate::ast::expression::{ComparisonKind, Expression, FunctionArgument};
    use crate::ast::literal::Literal;
    use crate::ast::node::Node;
    use crate::ast::statement::{AssignmentKind, ConditionalPath, Statement};
    use crate::ast::types::{BaseType, Type, TypeName};
    use crate::parser::test_utils::{run_test, run_tests};

    #[test]
    fn test_define_statement() {
        let src = "int x = 1";
        let expected = Statement::VariableDefinition {
            type_node: Node::new(
                Type::new(Node::new(TypeName::BaseType(BaseType::Int), 0..3), false),
                0..3,
            ),
            name: Node::new("x", 4..5),
            initial_value: Some(Node::new(Expression::Literal(Literal::Integer(1)), 8..9)),
        };

        run_test(src, expected);
    }

    #[test]
    fn test_return_statement() {
        let data = vec![
            ("Return", Statement::Return(None)),
            (
                "Return 1",
                Statement::Return(Some(Node::new(
                    Expression::Literal(Literal::Integer(1)),
                    7..8,
                ))),
            ),
        ];

        run_tests(data);
    }

    #[test]
    fn test_assignment_statement() {
        let lhs = Node::new(Expression::Identifier("x"), 0..1);

        let rhs = Node::new(Expression::Literal(Literal::Integer(1)), 5..6);

        let data = vec![
            (
                "x = 1",
                Statement::Assignment {
                    lhs: lhs.clone(),
                    kind: Node::new(AssignmentKind::Normal, 2..3),
                    rhs: Node::new(Expression::Literal(Literal::Integer(1)), 4..5),
                },
            ),
            (
                "x += 1",
                Statement::Assignment {
                    lhs: lhs.clone(),
                    kind: Node::new(AssignmentKind::Addition, 2..4),
                    rhs: rhs.clone(),
                },
            ),
            (
                "x -= 1",
                Statement::Assignment {
                    lhs: lhs.clone(),
                    kind: Node::new(AssignmentKind::Subtraction, 2..4),
                    rhs: rhs.clone(),
                },
            ),
            (
                "x *= 1",
                Statement::Assignment {
                    lhs: lhs.clone(),
                    kind: Node::new(AssignmentKind::Multiplication, 2..4),
                    rhs: rhs.clone(),
                },
            ),
            (
                "x /= 1",
                Statement::Assignment {
                    lhs: lhs.clone(),
                    kind: Node::new(AssignmentKind::Division, 2..4),
                    rhs: rhs.clone(),
                },
            ),
            (
                "x %= 1",
                Statement::Assignment {
                    lhs: lhs.clone(),
                    kind: Node::new(AssignmentKind::Modulus, 2..4),
                    rhs: rhs.clone(),
                },
            ),
            (
                "Pages[i] = Pages[j]",
                Statement::Assignment {
                    lhs: Node::new(
                        Expression::ArrayAccess {
                            array: Node::new(Expression::Identifier("Pages"), 0..5),
                            index: Node::new(Expression::Identifier("i"), 6..7),
                        },
                        0..8,
                    ),
                    kind: Node::new(AssignmentKind::Normal, 9..10),
                    rhs: Node::new(
                        Expression::ArrayAccess {
                            array: Node::new(Expression::Identifier("Pages"), 11..16),
                            index: Node::new(Expression::Identifier("j"), 17..18),
                        },
                        11..19,
                    ),
                },
            ),
        ];

        run_tests(data);
    }

    #[test]
    fn test_if_statement() {
        let data = vec![
            (
                "If (true) EndIf",
                Statement::If {
                    if_path: Node::new(
                        ConditionalPath {
                            condition: Node::new(Expression::Literal(Literal::Boolean(true)), 3..9),
                            statements: None,
                        },
                        3..9,
                    ),
                    other_paths: None,
                    else_path: None,
                },
            ),
            (
                "If (true) ElseIf (true) ElseIf (false) Else EndIf",
                Statement::If {
                    if_path: Node::new(
                        ConditionalPath {
                            condition: Node::new(Expression::Literal(Literal::Boolean(true)), 3..9),
                            statements: None,
                        },
                        3..9,
                    ),
                    other_paths: Some(vec![
                        Node::new(
                            ConditionalPath {
                                condition: Node::new(
                                    Expression::Literal(Literal::Boolean(true)),
                                    17..23,
                                ),
                                statements: None,
                            },
                            17..23,
                        ),
                        Node::new(
                            ConditionalPath {
                                condition: Node::new(
                                    Expression::Literal(Literal::Boolean(false)),
                                    31..38,
                                ),
                                statements: None,
                            },
                            31..38,
                        ),
                    ]),
                    else_path: None,
                },
            ),
            (
                r#"if x == 0
    Return y
elseif x == 1
    Return y
else
    Return y
endif"#,
                Statement::If {
                    if_path: Node::new(
                        ConditionalPath::new(
                            Node::new(
                                Expression::Comparison {
                                    lhs: Node::new(Expression::Identifier("x"), 3..4),
                                    kind: ComparisonKind::EqualTo,
                                    rhs: Node::new(Expression::Literal(Literal::Integer(0)), 8..9),
                                },
                                3..9,
                            ),
                            Some(vec![Node::new(
                                Statement::Return(Some(Node::new(
                                    Expression::Identifier("y"),
                                    21..22,
                                ))),
                                14..22,
                            )]),
                        ),
                        3..22,
                    ),
                    other_paths: Some(vec![Node::new(
                        ConditionalPath::new(
                            Node::new(
                                Expression::Comparison {
                                    lhs: Node::new(Expression::Identifier("x"), 30..31),
                                    kind: ComparisonKind::EqualTo,
                                    rhs: Node::new(
                                        Expression::Literal(Literal::Integer(1)),
                                        35..36,
                                    ),
                                },
                                30..36,
                            ),
                            Some(vec![Node::new(
                                Statement::Return(Some(Node::new(
                                    Expression::Identifier("y"),
                                    48..49,
                                ))),
                                41..49,
                            )]),
                        ),
                        30..49,
                    )]),
                    else_path: Some(vec![Node::new(
                        Statement::Return(Some(Node::new(Expression::Identifier("y"), 66..67))),
                        59..67,
                    )]),
                },
            ),
        ];

        run_tests(data);
    }

    #[test]
    fn test_while_statement() {
        let src = "While (true) EndWhile";
        let expected = Statement::While(Node::new(
            ConditionalPath {
                condition: Node::new(Expression::Literal(Literal::Boolean(true)), 6..12),
                statements: None,
            },
            6..12,
        ));

        run_test(src, expected);
    }

    #[test]
    fn test_expression_statement() {
        let src = "Debug.Trace(msg)";
        let expected = Statement::Expression(Node::new(
            Expression::MemberAccess {
                lhs: Node::new(Expression::Identifier("Debug"), 0..5),
                rhs: Node::new(
                    Expression::FunctionCall {
                        name: Node::new(Expression::Identifier("Trace"), 6..11),
                        arguments: Some(vec![Node::new(
                            FunctionArgument::Positional(Node::new(
                                Expression::Identifier("msg"),
                                12..15,
                            )),
                            12..15,
                        )]),
                    },
                    6..16,
                ),
            },
            0..16,
        ));

        run_test(src, expected);
    }
}
