# Cifrs

Pronounced as cyphers, is a framework detection library used by Doctavious to detect static-site generators as part of its zero-configuration build system.

## Framework Detection

Framework detection utility.


Detects which framework a specific website is using. The framework's build/dev commands, directories and server port are also returned.

The following frameworks are detected:

Static site generators: Gatsby, Hugo, Jekyll, Next.js, Nuxt, Hexo, Gridsome, Docusaurus, Eleventy, Middleman, Phenomic, React-static, Stencil, Vuepress, Assemble, DocPad, Harp, Metalsmith, Roots, Wintersmith
Front-end frameworks: create-react-app, Vue, Sapper, Angular, Ember, Svelte, Expo, Quasar
Build tools: Parcel, Brunch, Grunt, Gulp
If you're looking for a way to run framework-info via CLI check the build-info project.


Old

Netlify Dev will attempt to detect the site generator or build command that you are using, and run these on your behalf, while adding other development utilities. If you have a JavaScript project, it looks for the best package.json script to run for you, using simple heuristics, so you can use the full flexibility of npm scripts.

Overriding framework detection: The number of frameworks which Netlify Dev can detect is growing, but if yours is not yet supported (contributions welcome!), you can instruct Netlify Dev to run the project on your behalf by declaring it in a [dev] block of your netlify.toml file.

## Skip Build Step

Some static projects do not require building. For example, a website with only HTML/CSS/JS source files can be served as-is.

In such cases, you should:

Specify "Other" as the framework preset
Enable the Override option for the Build Command
Leave the Build Command empty
This prevents running the build, and your content is served directly.


## Commands

should there be a command to get / show framework detection?
