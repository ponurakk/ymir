use std::fmt::Display;

pub enum Sorting {
    Name,
    Size,
    Commits,
    CreationDate,
    ModificationDate,
    Loc,
}

impl Sorting {
    pub const fn next(&self) -> Self {
        match *self {
            Self::Name => Self::Size,
            Self::Size => Self::Commits,
            Self::Commits => Self::CreationDate,
            Self::CreationDate => Self::ModificationDate,
            Self::ModificationDate => Self::Loc,
            Self::Loc => Self::Name,
        }
    }

    pub const fn previous(&self) -> Self {
        match *self {
            Self::Loc => Self::ModificationDate,
            Self::ModificationDate => Self::CreationDate,
            Self::CreationDate => Self::Commits,
            Self::Commits => Self::Size,
            Self::Size => Self::Name,
            Self::Name => Self::Loc,
        }
    }
}

impl Display for Sorting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Name => write!(f, "Name"),
            Self::Size => write!(f, "Size"),
            Self::Commits => write!(f, "Commits"),
            Self::CreationDate => write!(f, "Creation Date"),
            Self::ModificationDate => write!(f, "Modification Date"),
            Self::Loc => write!(f, "Lines of Code"),
        }
    }
}

pub enum Filter {
    All,
    Owned,
    NotOwned,
    HasRemote,
    NoRemote,
}

impl Filter {
    pub const fn next(&self) -> Self {
        match self {
            Self::All => Self::Owned,
            Self::Owned => Self::NotOwned,
            Self::NotOwned => Self::HasRemote,
            Self::HasRemote => Self::NoRemote,
            Self::NoRemote => Self::All,
        }
    }

    pub const fn previous(&self) -> Self {
        match self {
            Self::NoRemote => Self::HasRemote,
            Self::HasRemote => Self::NotOwned,
            Self::NotOwned => Self::Owned,
            Self::Owned => Self::All,
            Self::All => Self::NoRemote,
        }
    }
}

impl Display for Filter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::All => write!(f, "All"),
            Self::Owned => write!(f, "Owned"),
            Self::NotOwned => write!(f, "Not Owned"),
            Self::HasRemote => write!(f, "Has Remote"),
            Self::NoRemote => write!(f, "No Remote"),
        }
    }
}
