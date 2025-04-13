use crate::{A11yNode, RoleSet, Tree, TreeCount, TreeTraversal};
use rayon::iter::ParallelIterator;

use std::fs;
use std::sync::OnceLock;

const REAL_FN: &str = "../../data/single-page-html-spec.json";

static REAL_JSON: OnceLock<String> = OnceLock::new();
static REAL_TREE_NODES: OnceLock<A11yNode> = OnceLock::new();
static REAL_TREE: OnceLock<Tree> = OnceLock::new();
static REAL_TREE_COUNT: OnceLock<TreeCount> = OnceLock::new();

fn real_data() -> &'static String {
    REAL_JSON.get_or_init(|| fs::read_to_string(REAL_FN).expect("Able to read file!"))
}
fn real_tree_nodes() -> &'static A11yNode {
    let data = real_data();
    REAL_TREE_NODES.get_or_init(|| serde_json::from_str(data).expect("Valid JSON!"))
}
fn real_tree() -> &'static Tree {
    let root_node = real_tree_nodes();
    REAL_TREE.get_or_init(|| {
        let mut t = Tree::from_root_node(root_node.clone());
        t.build_rolesets();
        t
    })
}
fn real_tree_count() -> &'static TreeCount {
    let root_node = real_tree_nodes();
    REAL_TREE_COUNT.get_or_init(|| {
        let mut tc = TreeCount::from_root_node(root_node.clone());
        tc.build_rolesets();
        tc
    })
}

macro_rules! validate_fn {
    ($name:ident, $fn1:ident, $fn2:ident) => {
        #[test]
        fn $name() {
            let rt = real_tree();
            let rtc = real_tree_count();

            assert_eq!(
                rt.$fn1(),
                rt.$fn2(),
                "{}::{} != {}::{}",
                std::any::type_name_of_val(rt),
                stringify!($fn1),
                std::any::type_name_of_val(rt),
                stringify!($fn2),
            );
            assert_eq!(
                rtc.$fn1(),
                rtc.$fn2(),
                "{}::{} != {}::{}",
                std::any::type_name_of_val(rtc),
                stringify!($fn1),
                std::any::type_name_of_val(rtc),
                stringify!($fn2),
            );
            assert_eq!(
                rt.$fn1(),
                rtc.$fn2(),
                "{}::{} != {}::{}",
                std::any::type_name_of_val(rt),
                stringify!($fn1),
                std::any::type_name_of_val(rtc),
                stringify!($fn2),
            );
        }
    };
}

macro_rules! validate_iter {
    ($name:ident, $fn1:ident, $fn2:ident) => {
        #[test]
        fn $name() {
            let rt = real_tree();
            let rtc = real_tree_count();
            let res1 = rt.$fn1().collect::<Vec<_>>();
            let res2 = rt.$fn2().collect::<Vec<_>>();
            let resc1 = rtc.$fn1().collect::<Vec<_>>();
            let resc2 = rtc.$fn2().collect::<Vec<_>>();

            assert_eq!(
                res1,
                res2,
                "{}::{} != {}::{}",
                std::any::type_name_of_val(rt),
                stringify!($fn1),
                std::any::type_name_of_val(rt),
                stringify!($fn2),
            );
            assert_eq!(
                resc1,
                resc2,
                "{}::{} != {}::{}",
                std::any::type_name_of_val(rtc),
                stringify!($fn1),
                std::any::type_name_of_val(rtc),
                stringify!($fn2),
            );
        }
    };
}

validate_fn!(validate_max_depth, max_depth, par_max_depth);
validate_fn!(validate_unique_roles, unique_roles, par_unique_roles);
validate_fn!(
    validate_unique_roles_precalc,
    par_unique_roles,
    unique_roles_roleset
);
validate_iter!(validate_leafs, iter_leafs, par_iter_leafs);

#[test]
fn validate_find_first() {
    let rt = real_tree();
    let rtc = real_tree_count();
    for role in RoleSet::ALL.role_iter() {
        let ff = rt.find_first(role);
        let par_ff = rt.par_find_first(role);
        let rs_ff = rt.find_first_roleset(role);
        let par_rs_ff = rt.par_find_first_roleset(role);
        let ffc = rtc.find_first(role);
        let par_ffc = rtc.par_find_first(role);
        let rs_ffc = rtc.find_first_roleset(role);
        let par_rs_ffc = rtc.par_find_first_roleset(role);
        assert_eq!(
            ff,
            par_ff,
            "{}::{} != {}::{}",
            std::any::type_name_of_val(rt),
            "find_first",
            std::any::type_name_of_val(rt),
            "par_find_first",
        );
        assert_eq!(
            ff,
            rs_ff,
            "{}::{} != {}::{}",
            std::any::type_name_of_val(rt),
            "find_first",
            std::any::type_name_of_val(rt),
            "find_first_roleset",
        );
        assert_eq!(
            ff,
            par_rs_ff,
            "{}::{} != {}::{}",
            std::any::type_name_of_val(rt),
            "find_first",
            std::any::type_name_of_val(rt),
            "par_find_first_roleset",
        );
        assert_eq!(
            ffc,
            rs_ffc,
            "{}::{} != {}::{}",
            std::any::type_name_of_val(rtc),
            "find_first",
            std::any::type_name_of_val(rtc),
            "find_first_roleset",
        );
        assert_eq!(
            ffc,
            par_ffc,
            "{}::{} != {}::{}",
            std::any::type_name_of_val(rtc),
            "find_first",
            std::any::type_name_of_val(rtc),
            "par_find_first",
        );
        assert_eq!(
            ffc,
            par_rs_ffc,
            "{}::{} != {}::{}",
            std::any::type_name_of_val(rtc),
            "find_first",
            std::any::type_name_of_val(rtc),
            "par_find_first_roleset",
        );
    }
}

#[test]
fn find_first_stack() {
    let rt = real_tree();
    let rtc = real_tree_count();
    for role in RoleSet::ALL.role_iter() {
        let ff = rt.find_first(role);
        let ffs = rt.find_first_stack(role);
        let ffc = rtc.find_first(role);
        let ffcs = rtc.find_first_stack(role);
        assert_eq!(
            ff,
            ffs,
            "{}::{} != {}::{}",
            std::any::type_name_of_val(rt),
            "find_first",
            std::any::type_name_of_val(rt),
            "find_first_stack",
        );
        assert_eq!(
            ffc,
            ffcs,
            "{}::{} != {}::{}",
            std::any::type_name_of_val(rtc),
            "find_first",
            std::any::type_name_of_val(rtc),
            "find_first_stack",
        );
    }
}

#[test]
fn validate_how_many() {
    let rt = real_tree();
    let rtc = real_tree_count();
    for role in RoleSet::ALL.role_iter() {
        let ff = rt.how_many(role);
        let par_ff = rt.par_how_many(role);
        let rs_ff = rt.how_many_roleset(role);
        let par_rs_ff = rt.par_how_many_roleset(role);
        let ffc = rtc.how_many(role);
        let par_ffc = rtc.par_how_many(role);
        let rs_ffc = rtc.how_many_roleset(role);
        let par_rs_ffc = rtc.par_how_many_roleset(role);
        assert_eq!(
            ff,
            par_ff,
            "{}::{} != {}::{} ({:?})",
            std::any::type_name_of_val(rt),
            "how_many",
            std::any::type_name_of_val(rt),
            "par_how_many",
            role,
        );
        assert_eq!(
            ff,
            rs_ff,
            "{}::{} != {}::{} ({:?})",
            std::any::type_name_of_val(rt),
            "how_many",
            std::any::type_name_of_val(rt),
            "how_many_roleset",
            role,
        );
        assert_eq!(
            ff,
            par_rs_ff,
            "{}::{} != {}::{} ({:?})",
            std::any::type_name_of_val(rt),
            "how_many",
            std::any::type_name_of_val(rt),
            "par_how_many_roleset",
            role,
        );
        assert_eq!(
            ffc,
            rs_ffc,
            "{}::{} != {}::{} ({:?})",
            std::any::type_name_of_val(rtc),
            "how_many",
            std::any::type_name_of_val(rtc),
            "how_many_roleset",
            role
        );
        assert_eq!(
            ffc,
            par_ffc,
            "{}::{} != {}::{} ({:?})",
            std::any::type_name_of_val(rtc),
            "how_many",
            std::any::type_name_of_val(rtc),
            "par_how_many",
            role,
        );
        assert_eq!(
            ffc,
            par_rs_ffc,
            "{}::{} != {}::{} ({:?})",
            std::any::type_name_of_val(rtc),
            "how_many",
            std::any::type_name_of_val(rtc),
            "par_how_many_roleset",
            role,
        );
    }
}
