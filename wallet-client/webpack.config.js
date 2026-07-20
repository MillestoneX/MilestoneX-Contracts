const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");

// Inline all JS into the HTML so the output is a single self-contained file.
// We achieve this by using HtmlWebpackPlugin with an inject strategy that
// places the <script> tag inline. The custom plugin below rewrites the emitted
// HTML so that the <script src="…"> tag is replaced with the bundle content.
class InlineScriptPlugin {
  apply(compiler) {
    compiler.hooks.emit.tapAsync("InlineScriptPlugin", (compilation, callback) => {
      const htmlAssets = Object.keys(compilation.assets).filter((name) =>
        name.endsWith(".html")
      );

      htmlAssets.forEach((htmlFile) => {
        let html = compilation.assets[htmlFile].source();

        // Find all <script src="...bundle.js..."> references and inline them.
        const scriptRegex = /<script\s+(?:[^>]*?\s)?src="([^"]+\.js)"[^>]*><\/script>/gi;
        let match;
        while ((match = scriptRegex.exec(html)) !== null) {
          const src = match[1];
          // The src may be relative; look it up in compilation assets.
          const assetKey = Object.keys(compilation.assets).find(
            (k) => k === src || k.endsWith("/" + src)
          );
          if (assetKey) {
            const scriptContent = compilation.assets[assetKey].source();
            html = html.replace(
              match[0],
              `<script>\n${scriptContent}\n</script>`
            );
            // Remove the JS file from emitted assets (we don't need a separate file).
            delete compilation.assets[assetKey];
          }
        }

        compilation.assets[htmlFile] = {
          source: () => html,
          size: () => html.length,
        };
      });

      callback();
    });
  }
}

module.exports = {
  entry: "./src/index.js",
  output: {
    path: path.resolve(__dirname, ".."),   // project root — overwrites wallet_connect.html
    filename: "wallet_connect.bundle.js",  // intermediate; InlineScriptPlugin deletes it
    clean: false,
  },
  resolve: {
    fallback: {
      // stellar-sdk pulls in Node built-ins; stub them for the browser.
      buffer: false,
      crypto: false,
      stream: false,
      path: false,
      fs: false,
      http: false,
      https: false,
      zlib: false,
      os: false,
      url: false,
      assert: false,
      util: false,
    },
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: "./src/template.html",
      filename: "wallet_connect.html",
      inject: "body",
      scriptLoading: "blocking",
    }),
    new InlineScriptPlugin(),
  ],
  devServer: {
    static: {
      directory: path.resolve(__dirname, ".."),
    },
    open: "/wallet_connect.html",
    port: 3000,
  },
  // Keep a reasonable bundle size for a browser page.
  performance: {
    hints: false,
  },
};
