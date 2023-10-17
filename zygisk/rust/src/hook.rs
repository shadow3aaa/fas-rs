/* Copyright 2023 shadow3aaa@gitbub.com
*
*  Licensed under the Apache License, Version 2.0 (the "License");
*  you may not use this file except in compliance with the License.
*  You may obtain a copy of the License at
*
*      http://www.apache.org/licenses/LICENSE-2.0
*
*  Unless required by applicable law or agreed to in writing, software
*  distributed under the License is distributed on an "AS IS" BASIS,
*  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
*  See the License for the specific language governing permissions and
*  limitations under the License. */
use std::{fs, path::Path, ptr};

use cpp_demangle::Symbol;
use dobby_api::{hook, resolve_func_addr, undo_hook, Address};
use goblin::Object;

use crate::error::{Error, Result};

pub struct SymbolHooker {
    symbols: Vec<String>,
}

impl SymbolHooker {
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Self> {
        let p = p.as_ref();
        let buffer = fs::read(p)?;

        let Object::Elf(lib) = Object::parse(&buffer)? else {
            return Err(Error::Other("Not an elf lib"));
        };

        if !lib.is_lib {
            return Err(Error::Other("Not an elf lib"))?;
        }

        let symbols = lib
            .dynstrtab
            .to_vec()?
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect();
        Ok(Self { symbols })
    }

    pub unsafe fn find_and_hook<S: AsRef<str>>(
        &self,
        s: S,
        replace_func: Address,
    ) -> Result<Address> {
        let symbol = self.find_symbol(s)?;
        let mut save_temp = ptr::null_mut();

        let _ = undo_hook(symbol);
        hook(symbol, replace_func, Some(&mut save_temp))?;

        Ok(save_temp)
    }

    fn find_symbol<S: AsRef<str>>(&self, s: S) -> Result<Address> {
        let s = s.as_ref();

        let symbol = self
            .symbols
            .iter()
            .filter_map(|sym| {
                let sym_de = Symbol::new(sym).ok()?;
                if sym_de.to_string().contains(s) {
                    Some(sym)
                } else {
                    None
                }
            })
            .min_by_key(|sym| sym.len())
            .ok_or(Error::Symbol)?;

        Ok(resolve_func_addr(None, symbol)?)
    }
}
