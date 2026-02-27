#!/usr/bin/env python3
"""
compose.py — Assemble a complete integration guide from local sections + shared docs.

Usage:
    python3 docs/scripts/compose.py <shared-docs-path> [manifest]

Arguments:
    shared-docs-path   Path to a local clone of the zombie-delete-docs repo.
                       The GitHub Action clones the repo before invoking this script.
                       See .github/workflows/build-guide.yml.
    manifest           Path to compose.yaml (default: docs/compose.yaml)

Output:
    Writes the composed markdown file to the repo root (filename from manifest).

Dependencies:
    PyYAML (pip install pyyaml)

Manifest directives:
    local:          Path relative to docs/sections/
    shared:         Path relative to zombie-delete-docs root
    merge:          Two files combined (base + append).
                    Paths use shared/ prefix for shared docs, bare for local.
    heading:        Top-level markdown heading injected before section content.
    preamble:       Paragraph injected after heading, before section content.
    strip_heading:  If true, remove the first ## heading from the section content.
                    Only strips headings at level 2 (##) or deeper. A # heading
                    will NOT be stripped (safety check — this likely indicates a
                    manifest misconfiguration). Default: false.
"""

import sys
import os
import re
import yaml


def github_anchor(heading_text: str) -> str:
    """
    Convert a markdown heading to a GitHub-style anchor slug.
    Rules: lowercase, strip markup, collapse whitespace to hyphens,
    drop non-alphanumeric except hyphens, strip leading/trailing hyphens.
    """
    # Remove the leading #s and strip
    text = re.sub(r'^#+\s*', '', heading_text).strip()
    # Lowercase
    text = text.lower()
    # Remove backticks and other inline markup
    text = re.sub(r'[`*_\[\]()]', '', text)
    # Replace whitespace and special chars with hyphens
    text = re.sub(r'[^\w-]', '-', text)
    # Collapse multiple hyphens
    text = re.sub(r'-+', '-', text)
    # Strip leading/trailing hyphens
    text = text.strip('-')
    return text


def read_section(filepath: str, label: str) -> str:
    """Read a markdown file, strip composability comments, return content."""
    if not os.path.isfile(filepath):
        print(f"ERROR: File not found: {filepath} (referenced as: {label})",
              file=sys.stderr)
        sys.exit(1)

    with open(filepath, "r", encoding="utf-8") as f:
        lines = f.readlines()

    # Strip HTML comment lines (composability metadata)
    filtered = [line for line in lines if not line.startswith("<!-- ")]

    # Strip leading blank lines
    while filtered and filtered[0].strip() == "":
        filtered.pop(0)

    content = "".join(filtered)

    if not content.endswith("\n"):
        content += "\n"

    return content


def first_heading(content: str) -> str | None:
    """Return the first markdown heading line, or None."""
    for line in content.split("\n"):
        if line.startswith("#"):
            return line
    return None


def strip_first_heading(content: str) -> str:
    """
    Remove the first ## (or deeper) heading and any trailing blank line.
    Refuses to strip a top-level # heading (safety: likely misconfiguration).
    """
    lines = content.split("\n")
    for i, line in enumerate(lines):
        if line.startswith("#"):
            if line.startswith("## ") or line.startswith("### ") or \
               line.startswith("#### "):
                lines.pop(i)
                # Remove trailing blank line after heading
                if i < len(lines) and lines[i].strip() == "":
                    lines.pop(i)
                return "\n".join(lines)
            else:
                # Top-level # heading — refuse to strip
                print(f"WARNING: strip_heading encountered a top-level "
                      f"# heading, skipping strip: {line}", file=sys.stderr)
                return content
    return content


def resolve_ref(ref: str, sections_dir: str, shared_path: str):
    """Resolve a reference to a filesystem path and label."""
    if ref.startswith("shared/"):
        rel = ref[len("shared/"):]
        return os.path.join(shared_path, rel), f"shared: {ref}"
    else:
        return os.path.join(sections_dir, ref), f"local: {ref}"


def source_filename(ref: str) -> str | None:
    """Extract the bare filename from a ref (e.g., 'shared/icp/foo.md' → 'foo.md')."""
    if ref and ref.endswith(".md"):
        return os.path.basename(ref)
    return None


def collapse_blank_lines(content: str) -> str:
    """Collapse 3+ consecutive blank lines to 2."""
    return re.sub(r"\n{4,}", "\n\n\n", content)


def main():
    if len(sys.argv) < 2:
        print("Usage: compose.py <shared-docs-path> [manifest]", file=sys.stderr)
        sys.exit(1)

    shared_path = sys.argv[1]
    manifest_path = sys.argv[2] if len(sys.argv) > 2 else "docs/compose.yaml"
    sections_dir = os.path.join(os.path.dirname(manifest_path), "sections")

    if not os.path.isfile(manifest_path):
        print(f"ERROR: Manifest not found: {manifest_path}", file=sys.stderr)
        sys.exit(1)

    if not os.path.isdir(shared_path):
        print(f"ERROR: Shared docs path not found: {shared_path}", file=sys.stderr)
        sys.exit(1)

    with open(manifest_path, "r", encoding="utf-8") as f:
        manifest = yaml.safe_load(f)

    title = manifest["title"]
    version = manifest["version"]
    output = manifest["output"]
    sections = manifest["sections"]

    print(f"Composing: {title} v{version}")
    print(f"Manifest:  {manifest_path}")
    print(f"Shared:    {shared_path}")
    print(f"Sections:  {len(sections)}")
    print(f"Output:    {output}")
    print()

    # ── Pass 1: Compose sections + build anchor map ───────────────────
    #
    # anchor_map: source filename → GitHub-style anchor slug.
    # Built dynamically from the actual headings inserted into the
    # composed document. Used in Pass 2 to rewrite cross-references.
    #
    # When a heading directive is present, the anchor comes from that
    # heading (since it replaces the file's own heading). Otherwise,
    # it comes from the file's first heading.

    parts = []
    anchor_map = {}  # "module-hash-pipeline.md" → "25-module-hash-deployment-patterns"

    for i, section in enumerate(sections):
        has_heading = "heading" in section
        has_preamble = "preamble" in section
        do_strip = section.get("strip_heading", False)

        # Inject heading
        if has_heading:
            parts.append(f"\n---\n\n{section['heading']}\n\n")

        # Inject preamble
        if has_preamble:
            parts.append(f"{section['preamble']}\n\n")

        # ── Read content ──────────────────────────────────────────────

        if "merge" in section:
            merge = section["merge"]
            base_path, base_label = resolve_ref(
                merge["base"], sections_dir, shared_path)
            append_path, append_label = resolve_ref(
                merge["append"], sections_dir, shared_path)

            flags = []
            if has_heading:
                flags.append("heading")
            if do_strip:
                flags.append("strip")
            flag_str = f" ({', '.join(flags)})" if flags else ""
            print(f"  [{i:2d}] merge: {base_label} + {append_label}{flag_str}")

            base_content = read_section(base_path, base_label)
            append_content = read_section(append_path, append_label)

            # Register anchors for both source files
            for ref, content in [(merge["base"], base_content),
                                 (merge["append"], append_content)]:
                fname = source_filename(ref)
                if fname and fname not in anchor_map:
                    if has_heading:
                        anchor_map[fname] = github_anchor(section["heading"])
                    else:
                        h = first_heading(content)
                        if h:
                            anchor_map[fname] = github_anchor(h)

            if do_strip:
                base_content = strip_first_heading(base_content)

            parts.append(base_content)
            parts.append("\n")
            parts.append(append_content)

        elif "local" in section:
            ref = section["local"]
            filepath = os.path.join(sections_dir, ref)
            label = f"local: {ref}"

            flags = []
            if has_heading:
                flags.append("heading")
            if do_strip:
                flags.append("strip")
            flag_str = f" ({', '.join(flags)})" if flags else ""
            print(f"  [{i:2d}] {label}{flag_str}")

            content = read_section(filepath, label)

            # Register anchor
            fname = source_filename(ref)
            if fname and fname not in anchor_map:
                if has_heading:
                    anchor_map[fname] = github_anchor(section["heading"])
                else:
                    h = first_heading(content)
                    if h:
                        anchor_map[fname] = github_anchor(h)

            if do_strip:
                content = strip_first_heading(content)
            parts.append(content)

        elif "shared" in section:
            ref = section["shared"]
            filepath = os.path.join(shared_path, ref)
            label = f"shared: {ref}"

            flags = []
            if has_heading:
                flags.append("heading")
            if do_strip:
                flags.append("strip")
            flag_str = f" ({', '.join(flags)})" if flags else ""
            print(f"  [{i:2d}] {label}{flag_str}")

            content = read_section(filepath, label)

            # Register anchor
            fname = source_filename(ref)
            if fname and fname not in anchor_map:
                if has_heading:
                    anchor_map[fname] = github_anchor(section["heading"])
                else:
                    h = first_heading(content)
                    if h:
                        anchor_map[fname] = github_anchor(h)

            if do_strip:
                content = strip_first_heading(content)
            parts.append(content)

        elif has_heading:
            print(f"  [{i:2d}] heading only: {section['heading']}")

        else:
            print(f"ERROR: Section {i} has no recognised type",
                  file=sys.stderr)
            sys.exit(1)

        parts.append("\n")

    # ── Pass 2: Post-processing ───────────────────────────────────────

    composed = "".join(parts)

    # Rewrite cross-references dynamically using the anchor map
    print()
    print("  Anchor map:")
    for fname, anchor in sorted(anchor_map.items()):
        print(f"    {fname} → #{anchor}")
        # Replace (filename.md) → (#anchor)
        composed = composed.replace(f"({fname})", f"(#{anchor})")
        # Replace (filename.md#fragment) → (#anchor)
        pattern = re.compile(re.escape(f"({fname}#") + r"([^)]+)\)")
        composed = pattern.sub(f"(#{anchor})", composed)

    # Check for any unresolved .md links
    residual = re.findall(r'\([^)]*\.md[^)]*\)', composed)
    if residual:
        print()
        print("  WARNING: Unresolved .md links in composed output:")
        for link in residual:
            print(f"    {link}")

    composed = collapse_blank_lines(composed)

    # Fix #2: only strip leading newlines, nothing else
    composed = composed.lstrip("\n")

    with open(output, "w", encoding="utf-8") as f:
        f.write(composed)

    line_count = composed.count("\n")
    print()
    print(f"Done: {output} ({line_count} lines)")


if __name__ == "__main__":
    main()
