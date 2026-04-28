const ALTERNATE_GREETING_MARKER = "<START_ALT>";

export function parseAlternateGreetingSegments(
    source: string | string[] | undefined | null,
): string[] {
    const text = Array.isArray(source)
        ? source.join(`\n${ALTERNATE_GREETING_MARKER}\n`)
        : source || "";

    return text
        .split(ALTERNATE_GREETING_MARKER)
        .map((segment) => segment.trim())
        .filter((segment) => segment.length > 0);
}

export function formatAlternateGreetingsForInput(
    value: string | string[] | undefined | null,
): string {
    const segments = parseAlternateGreetingSegments(value);
    return serializeAlternateGreetingsValue(segments);
}

export function serializeAlternateGreetingsValue(
    value: string | string[] | undefined | null,
): string {
    const segments = parseAlternateGreetingSegments(value);
    if (!segments.length) {
        return "";
    }

    return segments
        .map((segment) => `${ALTERNATE_GREETING_MARKER}\n${segment}`)
        .join("\n");
}

export function parseTagsValue(value: string | string[] | undefined | null): string[] {
    if (Array.isArray(value)) {
        return value.map((tag) => tag.trim()).filter((tag) => tag.length > 0);
    }

    return (value || "")
        .split(/,|\n/)
        .map((tag) => tag.trim())
        .filter((tag) => tag.length > 0);
}

export function serializeTagsValue(value: string | string[] | undefined | null): string {
    return parseTagsValue(value).join(",");
}
