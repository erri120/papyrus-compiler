#[derive(PartialEq, Debug)]
pub enum KeywordKind {
    Auto,
    AutoReadOnly,
    BetaOnly,
    Bool,
    Const,
    CustomEvent,
    CustomEventName,
    DebugOnly,
    Else,
    ElseIf,
    EndEvent,
    EndFunction,
    EndGroup,
    EndIf,
    EndProperty,
    EndState,
    EndStruct,
    EndWhile,
    Event,
    Extends,
    False,
    Float,
    Function,
    Global,
    Group,
    If,
    Import,
    Int,
    Length,
    Native,
    New,
    None,
    Property,
    Return,
    ScriptName,
    ScriptEventName,
    State,
    String,
    Struct,
    StructVarName,
    True,
    Var,
    While,
}
