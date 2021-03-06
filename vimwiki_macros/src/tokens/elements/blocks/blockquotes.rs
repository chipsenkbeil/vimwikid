use crate::tokens::{utils::root_crate, Tokenize, TokenizeContext};
use proc_macro2::TokenStream;
use quote::quote;
use vimwiki_core::Blockquote;

impl_tokenize!(tokenize_blockquote, Blockquote<'a>, 'a);
fn tokenize_blockquote(
    ctx: &TokenizeContext,
    blockquote: &Blockquote,
) -> TokenStream {
    let root = root_crate();
    let lines = blockquote.lines.iter().map(|x| do_tokenize!(ctx, x));
    quote! {
        #root::Blockquote::new(::std::vec![#(#lines),*])
    }
}
