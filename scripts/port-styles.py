#!/usr/bin/env python3
"""Copy a web/ component's <style> block into a global stylesheet (ADR 0032).

Unwraps :global(), drops comments and prefixes every selector with the root class, unless it
already starts with it.

usage: scripts/port-styles.py <web component.svelte> <.root-class> <out.css> [--append]
"""
import re
import sys
from pathlib import Path


def scope_selectors(selectors: str, root: str) -> str:
    out = []
    for s in (part.strip() for part in selectors.split(",")):
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


def main() -> None:
    src, root, out = sys.argv[1:4]
    text = Path(src).read_text()
    css = text[text.index("<style>") + 7 : text.rindex("</style>")]
    web_path = src[src.index("web/") :] if "web/" in src else src
    block = f"/* Ported from {web_path} (ADR 0032, scripts/port-styles.py) */\n{port(css, root)}\n"
    mode = "a" if "--append" in sys.argv else "w"
    with open(out, mode) as f:
        f.write(("\n" if mode == "a" else "") + block)
    print(f"{out}: {'appended' if mode == 'a' else 'written'} {web_path}")


if __name__ == "__main__":
    main()
