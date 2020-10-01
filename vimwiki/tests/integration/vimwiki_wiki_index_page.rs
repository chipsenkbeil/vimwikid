use super::{
    fixtures::VimwikiFile,
    utils::{blank_line, compare_page_components},
};
use std::convert::TryInto;
use vimwiki::{components::*, RawStr, Region, LC};
use vimwiki_macros::*;

#[test]
fn test() {
    let page: LC<Page> =
        RawStr::Vimwiki(&VimwikiFile::VimwikiWikiIndex.load().unwrap())
            .try_into()
            .unwrap();
    let expected = vec![
        vimwiki_header!("= Vimwiki Wiki =")
            .take_with_region(Region::from((1, 1, 1, 17)))
            .into(),
        blank_line()
            .take_with_region(Region::from((2, 1, 2, 1))),
        vimwiki_paragraph!("*Welcome to the Vimwiki wiki!*")
            .take_with_region(Region::from((3, 1, 3, 31)))
            .into(),
        blank_line()
            .take_with_region(Region::from((4, 1, 4, 1))),
        vimwiki_header!("== Official Repositories ==")
            .take_with_region(Region::from((5, 1, 5, 28)))
            .into(),
        blank_line()
            .take_with_region(Region::from((6, 1, 6, 1))),
        vimwiki_paragraph!("Here are links to the official Vimwiki repositories:")
            .take_with_region(Region::from((7, 1, 7, 53)))
            .into(),
        blank_line()
            .take_with_region(Region::from((8, 1, 8, 1))),
        vimwiki_list! {r#"
            - [[https://github.com/vimwiki/vimwiki|Vimwiki]]
            - [[https://vimwiki.github.io/vimwikiwiki/|Vimwiki Wiki]] (GitHub pages site built using Vimwiki)
                - [[https://github.com/vimwiki/vimwikiwiki|source repository]]
            - [[https://github.com/vimwiki/utils|Utilities]]
            - [[https://github.com/vimwiki/testwikis|Test wikis]] - sample wikis in each of
              the 3 syntax variants. Used for testing and development.
        "#}
            .take_with_region(Region::from((9, 1, 14, 59)))
            .into(),
        blank_line()
            .take_with_region(Region::from((15, 1, 15, 1))),
        vimwiki_header!("== Wiki Pages ==")
            .take_with_region(Region::from((16, 1, 16, 17)))
            .into(),
        blank_line()
            .take_with_region(Region::from((17, 1, 17, 1))),
        vimwiki_paragraph!("Checkout these pages for additional information and tips!")
            .take_with_region(Region::from((18, 1, 18, 58)))
            .into(),
        blank_line()
            .take_with_region(Region::from((19, 1, 19, 1))),
        vimwiki_list! {r#"
            - [[Tips and Snips]]
            - [[Related Tools]]
            - [[Troubleshooting]]
        "#}
            .take_with_region(Region::from((20, 1, 22, 22)))
            .into(),
        blank_line()
            .take_with_region(Region::from((23, 1, 23, 1))),
        vimwiki_header!("== Chat/Forums ==")
            .take_with_region(Region::from((24, 1, 24, 18)))
            .into(),
        blank_line()
            .take_with_region(Region::from((25, 1, 25, 1))),
        vimwiki_list! {r#"
            - [[https://groups.google.com/forum/#!forum/vimwiki|Google Vimwiki group]]
            - [[https://webchat.freenode.net/?channels=#vimwiki|Freenode Webchat]]
        "#}
            .take_with_region(Region::from((26, 1, 27, 71)))
            .into(),
        blank_line()
            .take_with_region(Region::from((28, 1, 28, 1))),
        vimwiki_header!("== Outdated Versions ==")
            .take_with_region(Region::from((29, 1, 29, 24)))
            .into(),
        blank_line()
            .take_with_region(Region::from((30, 1, 30, 1))),
        vimwiki_paragraph! {r#"
            These links point to some old versions of Vimwiki that are no longer maintained.
            The original Vimwiki was hosted on Google Code which has since shutdown.
        "#}
            .take_with_region(Region::from((31, 1, 32, 73)))
            .into(),
        blank_line()
            .take_with_region(Region::from((33, 1, 33, 1))),
        vimwiki_list! {r#"
                - [[https://code.google.com/archive/p/vimwiki/|Google Code Archive]]
                - [[https://github.com/vimwiki-backup/vimwiki|Google Code Backup on Github]]
                - [[https://www.vim.org/scripts/script.php?script_id=2226|Vimwiki on vim.org]]
            "#}
            .take_with_region(Region::from((34, 1, 36, 79)))
            .into(),
        blank_line()
            .take_with_region(Region::from((37, 1, 37, 1))),
        vimwiki_header!("== Related Projects ==")
            .take_with_region(Region::from((38, 1, 38, 23)))
            .into(),
        blank_line()
            .take_with_region(Region::from((39, 1, 39, 1))),
        vimwiki_list! {r#"
            - [[https://github.com/lervag/wiki.vim|wiki.vim]]
            - [[https://github.com/fcpg/vim-waikiki|vim-waikiki]]
            - [[https://github.com/jceb/vim-orgmode|vim-orgmode]]
            - [[https://github.com/tbabej/taskwiki|taskwiki]]
            - [[https://github.com/xolox/vim-notes|vim-notes]]
        "#}
            .take_with_region(Region::from((40, 1, 44, 51)))
            .into(),
    ];

    compare_page_components(&page.components, &expected);
}