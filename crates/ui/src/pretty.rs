use annotate_snippets::{Annotation, AnnotationKind, Group, Level, Snippet};
use jjpwrgem_parse::error::diagnostics::{Context, Diagnostic, Patch, Source};

pub fn patch_to_patch<'a>(patch: Patch<'a>) -> annotate_snippets::Patch<'a> {
    annotate_snippets::Patch::new(patch.span, patch.replacement)
}

pub fn source_to_snippet<'a, T: Clone>(val: Source<'a>) -> Snippet<'a, T> {
    let (source, path) = match val {
        Source::Stdin(src) => (src, "stdin"),
        Source::File { source, path } => (
            source,
            path.to_str()
                .expect("diagnostic paths should be valid utf8"),
        ),
    };
    Snippet::source(source).path(path)
}

pub fn context_to_annotation<'a>(ctx: Context<'a>) -> Annotation<'a> {
    let Context {
        message,
        span,
        source: _,
    } = ctx;
    AnnotationKind::Context.span(span).label(message)
}

pub fn report<'a>(
    Diagnostic {
        message,
        context,
        patches,
        source,
        range,
    }: Diagnostic<'a>,
) -> Vec<Group<'a>> {
    let annotations = if let Some(range) = range {
        std::iter::once(AnnotationKind::Primary.span(range))
            .chain(context.into_iter().map(context_to_annotation))
            .collect()
    } else {
        vec![]
    };

    let error_group = Level::ERROR
        .primary_title(message)
        .element(source_to_snippet(source).annotations(annotations));
    let patch_group = patches.into_iter().map(|patch| {
        Level::HELP
            .primary_title(patch.message.clone())
            .element(source_to_snippet(source).patches(vec![patch_to_patch(patch)]))
    });

    std::iter::once(error_group).chain(patch_group).collect()
}
