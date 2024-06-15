pub enum Result<I, O, E> {
    Ok { output: O, rest_of_input: I },
    RecoverableError,
    UnrecoverableError(E),
}
