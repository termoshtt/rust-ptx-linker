use std::collections::BTreeSet;
use std::ffi::CStr;

use llvm_sys::core::*;
use llvm_sys::prelude::*;
use llvm_sys::LLVMValueKind;

use crate::llvm::CallVisitor;

pub const SYSCALLS: &[&str] = &["vprintf", "__assertfail", "malloc", "free"];

pub struct FindExternalReferencesPass {
    references: BTreeSet<String>,
}

impl FindExternalReferencesPass {
    pub fn new() -> Self {
        FindExternalReferencesPass {
            references: BTreeSet::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.references.len()
    }

    pub fn references(self) -> Vec<String> {
        self.references.into_iter().collect()
    }
}

impl CallVisitor for FindExternalReferencesPass {
    fn visit_call(&mut self, _caller: LLVMValueRef, callee: LLVMValueRef) -> bool {
        let callee_name = unsafe {
            let mut callee_name_len = 0;

            CStr::from_ptr(LLVMGetValueName2(callee, &mut callee_name_len)).to_string_lossy()
        };

        let is_declaration = unsafe {
            LLVMGetValueKind(callee) == LLVMValueKind::LLVMFunctionValueKind
                && LLVMIsDeclaration(callee) == 1
        };

        let is_intrinsic = callee_name.starts_with("llvm.");
        let is_syscall = SYSCALLS.contains(&callee_name.as_ref());

        if is_declaration && !is_intrinsic && !is_syscall {
            self.references.insert(callee_name.into());
        }

        false
    }
}
