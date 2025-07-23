import ReactMarkdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import rehypeKatex from "rehype-katex";
import rehypeRaw from "rehype-raw";
import remarkBreaks from "remark-breaks";
import remarkEmoji from "remark-emoji";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";

// Import KaTeX CSS for math rendering
import "katex/dist/katex.min.css";
// Import highlight.js theme
import "highlight.js/styles/github.css";
import { useUserAppearanceTheme } from "../../store";

// Import custom markdown styles

interface EnhancedMarkdownRendererProps {
  content: string;
  className?: string;
}

export function EnhancedMarkdownRenderer({
  content,
  className,
}: EnhancedMarkdownRendererProps) {
  const resolvedTheme = useUserAppearanceTheme();

  return (
    <div className={`classless ${resolvedTheme} ${className || ""}`}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm, remarkBreaks, remarkMath, remarkEmoji]}
        rehypePlugins={[
          [
            rehypeHighlight,
            {
              detect: true,
              ignoreMissing: true,
            },
          ],
          rehypeRaw,
          rehypeKatex,
        ]}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
}
