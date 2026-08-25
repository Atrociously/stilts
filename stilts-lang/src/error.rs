pub enum ParentError<Ref> {
    Circular { parentage: Vec<Ref> },
    Exists { child: Ref, existing_parent: Ref },
}
