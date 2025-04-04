import { defineConfig } from "@rspack/cli";
import { rspack } from "@rspack/core";

export default defineConfig({
  entry: {
    main: "./web/index.js",
  },
  module: {
    rules: [
      {
        test: /\.js$/,
        use: [
          {
            loader: "builtin:swc-loader",
            options: {
              jsc: {
                parser: {
                  syntax: "ecmascript",
                },
              },
            },
          },
        ],
      },
      {
        test: /\.css$/,
        use: ["postcss-loader"],
        type: "css",
      },
    ],
  },
  optimization: {
    minimizer: [new rspack.SwcJsMinimizerRspackPlugin()],
  },
  experiments: {
    css: true,
  },
});
