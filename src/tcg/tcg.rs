#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ValueId(u32);

pub struct Value {
    pub ty: ValueType,
}

#[derive(Copy, Clone, Debug)]
pub enum ValueType {
    I32,
    I64,
    Ptr,
}

#[derive(Debug)]
pub enum IRInst {
    Add {
        dst: ValueId,
        a: ValueId,
        b: ValueId,
    },
    AddI {
        dst: ValueId,
        a: ValueId,
        imm: i64,
    },
    Sub {
        dst: ValueId,
        a: ValueId,
        b: ValueId,
    },

    Load {
        dst: ValueId,
        addr: ValueId,
    },
    Store {
        addr: ValueId,
        val: ValueId,
    },

    Br {
        target: u64,
    },
    BrCond {
        cond: ValueId,
        target: u64,
    },

    Exit,
}
