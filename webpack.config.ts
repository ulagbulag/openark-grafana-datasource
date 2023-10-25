import CopyWebpackPlugin from 'copy-webpack-plugin';
import path from 'path';
import type { Configuration } from 'webpack';
import { merge } from 'webpack-merge';
import PermissionsOutputPlugin from 'webpack-permissions-plugin';
import grafanaConfig from './.config/webpack/webpack.config';

const config = async (env): Promise<Configuration> => {
  const baseConfig = await grafanaConfig(env);

  return merge(baseConfig, {
    output: {
      publicPath: '/',
      uniqueName: undefined,
    },

    plugins: [
      new CopyWebpackPlugin({
        patterns: [
          {
            from: '../target/release/gpx_openark',
            to: './gpx_openark_linux_amd64',
            force: true,
            toType: "file",
          },
          {
            from: '../provisioning/.gitkeep',
            to: './.gitkeep',
            force: true,
            toType: "file",
          },
        ],
      }),
      new PermissionsOutputPlugin({
        buildFiles: [
          {
            path: path.resolve(__dirname, 'dist/gpx_openark_linux_amd64'),
            fileMode: '755'
          },
        ],
      }),
    ],
  });
};

export default config;
