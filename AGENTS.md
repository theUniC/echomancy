# Agent Instructions for Echomancy

## Project Overview

**Echomancy** is a Next.js 16 application using the App Router with React 19, TypeScript, and Tailwind CSS v4. This is currently a fresh project bootstrapped with `create-next-app`.

## Tech Stack & Key Dependencies

- **Next.js 16.0.7** with App Router (`src/app/` directory structure)
- **React 19.2.0** with **React Compiler** enabled (`reactCompiler: true` in `next.config.ts`)
- **TypeScript 5** with strict mode enabled
- **Tailwind CSS v4** (new version with PostCSS plugin `@tailwindcss/postcss`)
- **Biome** for linting and formatting (replaces ESLint + Prettier)

## Development Workflow

### Commands
```bash
npm run dev      # Start development server (localhost:3000)
npm run build    # Production build
npm run start    # Run production build
npm run lint     # Run Biome linter checks
npm run format   # Format code with Biome
```

### Code Quality
- Use **Biome** for all linting and formatting tasks
- Run `npm run format` before committing to ensure consistent code style
- Biome is configured in `biome.json` with Next.js and React recommended rules

## Project Structure

```
src/
  app/
    layout.tsx       # Root layout with Geist fonts
    page.tsx         # Home page
    globals.css      # Global styles with Tailwind v4 @theme
public/             # Static assets
```

## Code Conventions

### TypeScript
- **Strict mode enabled** - all type safety features are on
- Use `@/*` path alias for imports from `src/` (e.g., `import { Foo } from "@/components/foo"`)
- Prefer explicit types for component props and function parameters
- Use `type` over `interface` for consistency with existing code

### Styling with Tailwind CSS v4
- **New Tailwind v4 syntax**: Use `@import "tailwindcss"` in CSS (see `globals.css`)
- **Inline themes**: Use `@theme inline { }` blocks for custom CSS variables
- Global design tokens defined in `:root` with CSS variables (`--background`, `--foreground`)
- Font variables: `--font-geist-sans` and `--font-geist-mono` configured via Next.js font optimization
- Dark mode uses `prefers-color-scheme` media query with CSS variable overrides
- Prefer utility classes over custom CSS

### React Patterns
- **React Compiler is enabled** - write idiomatic React, avoid manual memoization unless profiling shows it's needed
- Use Server Components by default (Next.js App Router convention)
- Add `"use client"` directive only when needed (state, effects, browser APIs)
- Prefer composition over prop drilling

### Font Loading
- Geist Sans and Geist Mono loaded via `next/font/google` in `layout.tsx`
- Font variables applied to `<body>` with template literals: `` `${geistSans.variable} ${geistMono.variable}` ``

## Important Configuration Notes

### Next.js Configuration
- React Compiler enabled in `next.config.ts` - this optimizes re-renders automatically
- Using TypeScript for config files (`.ts` not `.js`)

### TypeScript Configuration
- Module resolution: `bundler` (modern Next.js default)
- JSX: `react-jsx` (new JSX transform)
- Path alias `@/*` maps to `./src/*`

### Biome Configuration
- Indent: 2 spaces
- VCS integration enabled with Git
- Ignores: `node_modules`, `.next`, `dist`, `build`
- Domains: Next.js and React rules enabled
- Auto-organize imports on save via `assist.actions.source.organizeImports`

## When Adding New Features

1. **Components**: Place in `src/app/` (colocated with routes) or create `src/components/` for shared components
2. **API Routes**: Use `src/app/api/` with route handlers
3. **Styling**: Continue using Tailwind utilities; extend design tokens in `globals.css` `@theme` block if needed
4. **Types**: Create `src/types/` for shared type definitions
5. **Utilities**: Create `src/lib/` or `src/utils/` for helper functions

## Testing & Validation

- Always run `npm run lint` to catch issues before committing
- Check dark mode behavior (defined via CSS variables in `globals.css`)
- Verify type safety with `tsc --noEmit` (already part of build process)

## Deployment

- Optimized for **Vercel** deployment (see README.md)
- Static assets go in `public/` directory
- Environment variables should use `NEXT_PUBLIC_` prefix for client-side access
