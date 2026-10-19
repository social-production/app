#!/usr/bin/env python3
"""Copy web/ components' <style> blocks into global stylesheets (ADR 0032).

Unwraps :global(), drops comments and prefixes every selector with the root class, unless it
already starts with it. Root "-" copies the selectors as they are (the app shell).

usage: scripts/port-styles.py                 regenerate everything listed in PORTS
       scripts/port-styles.py <web component.svelte> <.root-class|-> <out.css> [--append]
"""
import re
import sys
from pathlib import Path


PORTS = Path(__file__).resolve().parent.parent / "ui/src/lib/components/styles/PORTS"


def split_top_level(selectors: str) -> list[str]:
    """Split on commas outside parentheses (":is(.a, .b)" stays whole)."""
    parts, depth, current = [], 0, ""
    for ch in selectors:
        depth += (ch == "(") - (ch == ")")
        if ch == "," and depth == 0:
            parts.append(current)
            current = ""
        else:
            current += ch
    return [*parts, current]


def scope_selectors(selectors: str, root: str) -> str:
    if root == "-":
        return selectors
    out = []
    for s in (part.strip() for part in split_top_level(selectors)):
        if not s:
            continue
        keep = s in (":root", "html", "body") or s.startswith((root, "from", "to")) or re.fullmatch(r"\d+%", s)
        out.append(s if keep else f"{root} {s}")
    return ", ".join(out)


def port(css: str, root: str) -> str:
    css = re.sub(r"/\*.*?\*/", "", css, flags=re.S)
    for _ in range(3):
        css = re.sub(r":global\(((?:[^()]|\([^()]*\))*)\)", r"\1", css)
    result, stack, pos = [], [], 0
    while pos < len(css):
        brace, close = css.find("{", pos), css.find("}", pos)
        if brace == -1 and close == -1:
            result.append(css[pos:])
            break
        if close != -1 and (brace == -1 or close < brace):
            result.append(css[pos : close + 1])
            if stack:
                stack.pop()
            pos = close + 1
            continue
        head = css[pos:brace]
        lead, sel = head[: len(head) - len(head.lstrip())], head.strip()
        if sel.startswith("@"):
            stack.append("keyframes" if "keyframes" in sel else "at")
        elif stack and stack[-1] == "keyframes":
            stack.append("frame")
        else:
            stack.append("rule")
            sel = scope_selectors(sel, root)
        result.append(f"{lead}{sel} {{")
        pos = brace + 1
    return "".join(result).strip()


def ported_block(src: str, root: str) -> str:
    text = Path(src).read_text()
    css = text[text.index("<style>") + 7 : text.rindex("</style>")]
    web_path = src[src.index("web/") :] if "web/" in src else src
    return f"/* Ported from {web_path} (ADR 0032, scripts/port-styles.py) */\n{port(css, root)}\n"


def regenerate_all() -> None:
    """PORTS lines: <out file in styles/> <root> <web path>; files are written in listed order."""
    web = PORTS.parents[6]
    outputs: dict[str, list[str]] = {}
    for line in PORTS.read_text().splitlines():
        if not line.strip() or line.startswith("#"):
            continue
        out, root, src = line.split()
        outputs.setdefault(out, []).append(ported_block(str(web / src), root))
    for out, blocks in outputs.items():
        target = PORTS.parent / out if "/" not in out else PORTS.parents[1] / out
        target.write_text("\n".join(blocks))
        print(f"{target.relative_to(PORTS.parents[5])}: {len(blocks)} component(s)")


def main() -> None:
    if len(sys.argv) == 1:
        regenerate_all()
        return
    src, root, out = sys.argv[1:4]
    block = ported_block(src, root)
    mode = "a" if "--append" in sys.argv else "w"
    with open(out, mode) as f:
        f.write(("\n" if mode == "a" else "") + block)
    print(f"{out}: {'appended' if mode == 'a' else 'written'}")


if __name__ == "__main__":
    main()
