#![recursion_limit = "128"]

use syn::{AttributeArgs, DeriveInput, parse_macro_input};
use proc_macro::TokenStream;
use quote::quote;
use std::iter;

enum Looping {
    Loop,
    Once,
}

impl Looping {
    // TODO: use From
    fn into(s: &str) -> ConfigOptions {
        ConfigOptions::L(match s.to_lowercase().as_str() {
            "loop" => Looping::Loop,
            "once" => Looping::Once,
            _ => panic!("Expected either `Loop` or `Once` as the first arg.")
        })
    }
}

impl Default for Looping {
    fn default() -> Self {
        Looping::Loop
    }
}

enum Mutability {
    DoNot,
    Modify,
}

impl Mutability {
    // TODO: use From
    fn into(s: &str) -> ConfigOptions {
        ConfigOptions::M(match s.to_lowercase().as_str() {
            "val" => Mutability::DoNot,
            "mut" => Mutability::Modify,
            _ => panic!("Expected either `Val` or `Mut` as the second arg.")
        })
    }
}

impl Default for Mutability {
    fn default() -> Self {
        Mutability::DoNot
    }
}

enum ImplIter {
    Yes,
    Into,
    No,
}

impl ImplIter {
    // TODO: use From
    fn into(s: &str) -> ConfigOptions {
        ConfigOptions::I(match s.to_lowercase().as_str() {
            "iter" => ImplIter::Yes,
            "none" => ImplIter::No,
            "into" => ImplIter::Into,
            _ => panic!("Expected either `Iter` or `Into` or `None` as the third arg.")
        })
    }
}

impl Default for ImplIter {
    fn default() -> Self {
        ImplIter::Yes
    }
}

#[derive(Default)]
struct Config {
    looping: Looping,
    mutability: Mutability,
    impl_iter: ImplIter,
}

enum ConfigOptions {
    L(Looping),
    M(Mutability),
    I(ImplIter),
}

impl Config {
    fn set(&mut self, opt: ConfigOptions) -> &Self {
        use self::ConfigOptions::*;
        match opt {
            L(l) => self.looping = l,
            M(m) => self.mutability = m,
            I(i) => self.impl_iter = i,
        }

        self
    }
}

/// #[sequence({*LOOP, ONCE}, {*VAL, MUT}, {*ITER, NONE, INTO}))]
/// 3 args, all of which are optional. Valid options shown above in braces,
/// defaults marked with '*'.

// TODO: multiple errors before panicking?
#[proc_macro_attribute]
pub fn sequence(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = syn::parse_macro_input!(attr as AttributeArgs);
    let mut config = Config::default();
    let mut num = 0;

    for nm in attr {
        match num {
            0..=2 => {
                if let syn::NestedMeta::Meta(syn::Meta::Path(w)) = nm {
                    let nom: String = w.get_ident().unwrap().to_string();

                    config.set(match num {
                        0 => Looping::into,
                        1 => Mutability::into,
                        2 => ImplIter::into,
                        _ => unreachable!(),
                    }(&nom));

                } else {
                    panic!("Invalid arg!")
                }
            },
            _ => {
                // TODO: Span this correctly
                panic!("#[sequence] accepts up to 3 args! (looping, mutability, impl_iter)")
            }
        }

        num += 1;
    }

    let item = parse_macro_input!(item as DeriveInput);
    let mut states = Vec::new();

    let enum_name = item.ident.clone();

    match item.data {
        syn::Data::Enum(ref data) => {
            if data.variants.iter().count() == 0 { panic!("We can't handle Enums with no variants!"); }
            data.variants.iter().for_each(|v| match v.fields {
                syn::Fields::Unit => { states.push(v.ident.clone()) },
                // TODO: Span this right (i.e. on the problematic variant)
                _ => panic!("We can only handle Enums with Unit Variants (no fields)!")
            })
        },
        _ => panic!("We can only generate StateSequence implementations for Enums!")
    }

    // for a .. d this'll give us (a, b), ... (c, d).
    let state_transitions = states.iter()
        .zip(states.iter().skip(1))
        .chain(match config.looping {
            Looping::Loop => iter::once((states.last().unwrap(), states.first().unwrap())),
            Looping::Once => iter::once((states.last().unwrap(), states.last().unwrap())),
        });

    let (from, to) = (state_transitions.clone().map(|(f, _)| f), state_transitions.clone().map(|(_, t)| t));

    let match_block = quote! {
        match *self {
            #( #from => #to, )*
        }
    };

    let trait_impl = match config.mutability {
        Mutability::DoNot => {
            quote! {
                impl crate::aoc::friends::StateSequence for self::#enum_name {
                    fn next(&self) -> Self {
                        use self::#enum_name::*;
                        #match_block
                    }
                }
            }
        },
        Mutability::Modify => {
            quote! {
                impl crate::aoc::friends::StateSequenceMutate for self::#enum_name {
                    fn next(&mut self) -> Self {
                        use self::#enum_name::*;
                        *self = #match_block;
                        (*self).clone()
                    }
                }
            }
        }
    };

    let next_fn_invocation = match config.mutability {
        Mutability::DoNot => quote! { crate::aoc::friends::StateSequence },
        Mutability::Modify => quote! { crate::aoc::friends::StateSequenceMutate },
    };

    let with_iter_impl = match config.impl_iter {
        ImplIter::Yes => {
            quote! {
                #trait_impl

                impl Iterator for self::#enum_name {
                    type Item = Self;
                    fn next(&mut self) -> Option<Self> {
                        Some(#next_fn_invocation::next(self))
                    }
                }
            }
        },
        ImplIter::Into => {
            let span = enum_name.span();
            let struct_name = syn::Ident::new(&format!("_{}{}", enum_name.to_string(), "_Iterator"), span);

            quote! {
                #trait_impl

                #[derive(Debug)]
                struct #struct_name<'a> {
                    inner: &'a mut #enum_name,
                    last: Option<#enum_name>,
                }

                impl self::#enum_name {
                    fn iter(&mut self) -> #struct_name {
                        let last = Some(self.clone());
                        #struct_name {
                            inner: self,
                            last,
                        }
                    }
                }

                impl<'a> Iterator for #struct_name<'a> {
                    type Item = self::#enum_name;

                    fn next(&mut self) -> Option<Self::Item> {
                        let n = #next_fn_invocation::next(self.inner);
                        let c = self.last;

                        self.last = if let Some(c) = c {
                            if c == n {
                                None
                            } else {
                                Some(n)
                            }
                        } else { None };

                        c
                    }
                }
            }
        },
        ImplIter::No => trait_impl
    };

    let all_together = quote! {
        #[derive(Clone, Copy, PartialEq, Eq)]
        #item
        #with_iter_impl
    };

    all_together.into()
}
