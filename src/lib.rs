use std::{borrow::Cow, collections::HashMap};

use itertools::Itertools;
use regex::Captures;

pub mod defaults;
pub mod options;

use defaults::{RE, SORTER};
use options::Options;

pub fn has_classes(file_contents: &str) -> bool {
    RE.is_match(file_contents)
}

pub fn sort_file_contents<'a>(file_contents: &'a str, options: &Options) -> Cow<'a, str> {
    RE.replace_all(file_contents, |caps: &Captures| {
        let classes = &caps[1];
        let sorted_classes = sort_classes(classes, options);

        caps[0].replace(classes, &sorted_classes)
    })
}

fn sort_classes(class_string: &str, options: &Options) -> String {
    let str_vec = if options.allow_duplicates {
        sort_classes_vec(class_string.split_ascii_whitespace())
    } else {
        sort_classes_vec(class_string.split_ascii_whitespace().unique())
    };

    let mut string = String::with_capacity(str_vec.len() * 2);

    for str in str_vec {
        string.push_str(str);
        string.push(' ')
    }

    string.pop();
    string
}

fn sort_classes_vec<'a>(classes: impl Iterator<Item = &'a str>) -> Vec<&'a str> {
    let enumerated_classes = classes.map(|class| ((class), SORTER.get(class)));

    let mut tailwind_classes: Vec<(&str, &usize)> = vec![];
    let mut custom_classes: Vec<&str> = vec![];

    let responsive_sizes = vec!["sm", "md", "lg", "xl", "2xl", "3xl", "4xl", "5xl", "6xl"];

    let mut responsive: HashMap<&str, Vec<&str>> = responsive_sizes
        .iter()
        .map(|size| (*size, vec![]))
        .collect();

    for (class, maybe_size) in enumerated_classes {
        match maybe_size {
            Some(size) => tailwind_classes.push((class, size)),
            None => match class.as_bytes() {
                [b's', b'm', b':', ..] => responsive.get_mut("sm").unwrap().push(class),
                [b'm', b'd', b':', ..] => responsive.get_mut("md").unwrap().push(class),
                [b'l', b'g', b':', ..] => responsive.get_mut("lg").unwrap().push(class),
                [b'x', b'l', b':', ..] => responsive.get_mut("xl").unwrap().push(class),
                [b'2', b'x', b'l', b':', ..] => responsive.get_mut("2xl").unwrap().push(class),
                [b'3', b'x', b'l', b':', ..] => responsive.get_mut("3xl").unwrap().push(class),
                [b'4', b'x', b'l', b':', ..] => responsive.get_mut("4xl").unwrap().push(class),
                [b'5', b'x', b'l', b':', ..] => responsive.get_mut("5xl").unwrap().push(class),
                [b'6', b'x', b'l', b':', ..] => responsive.get_mut("6xl").unwrap().push(class),
                _ => custom_classes.push(class),
            },
        }
    }

    tailwind_classes.sort_by_key(|&(_class, class_placement)| class_placement);

    let sorted_tailwind_classes: Vec<&str> = tailwind_classes
        .into_iter()
        .map(|(class, _index)| class)
        .collect();

    let mut sorted_responsive_classes = vec![];

    for key in responsive_sizes {
        let (mut sorted_classes, new_custom_classes) = sort_responsive_classes(
            responsive.remove(key).unwrap(),
            custom_classes,
            key.len() + 1,
        );

        sorted_responsive_classes.append(&mut sorted_classes);
        custom_classes = new_custom_classes
    }

    [
        &sorted_tailwind_classes[..],
        &sorted_responsive_classes[..],
        &custom_classes[..],
    ]
    .concat()
}

fn sort_responsive_classes<'a>(
    classes: Vec<&'a str>,
    mut custom_classes: Vec<&'a str>,
    class_after: usize,
) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut tailwind_classes = Vec::with_capacity(classes.len());

    for class in classes {
        match SORTER.get(&class[class_after..]) {
            Some(class_placement) => tailwind_classes.push((class, class_placement)),
            None => custom_classes.push(class),
        }
    }

    tailwind_classes.sort_by_key(|&(_class, class_placement)| class_placement);

    let sorted_classes = tailwind_classes
        .into_iter()
        .map(|(class, _index)| class)
        .collect();

    (sorted_classes, custom_classes)
}

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn test_sort_classes_vec() {
    assert_eq!(
        sort_classes_vec(
            vec![
                "inline",
                "inline-block",
                "random-class",
                "py-2",
                "justify-end",
                "px-2",
                "flex"
            ]
            .into_iter()
        ),
        vec![
            "inline-block",
            "inline",
            "flex",
            "justify-end",
            "py-2",
            "px-2",
            "random-class",
        ]
    )
}
