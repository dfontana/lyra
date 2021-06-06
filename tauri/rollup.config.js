import json from '@rollup/plugin-json';
import {terser} from 'rollup-plugin-terser';
import postcss from 'rollup-plugin-postcss';
import { nodeResolve } from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';
import replace from '@rollup/plugin-replace';
import babel from '@rollup/plugin-babel';
import copy from 'rollup-plugin-copy'
import del from 'rollup-plugin-delete'

export default {
  input: 'src-js/js/index.js',
  output: [
    {
      file: 'src-js/dist/index.js',
      format: 'iife',
      sourcemap: 'inline',
      plugins: [terser()]
    }
  ],
  plugins: [
    json(),
    del({ targets: 'src-js/dist/*', force: true }),
    replace({
      'process.env.NODE_ENV': JSON.stringify('production')
    }),
    nodeResolve({
      extensions: ['.js', '.jsx']
    }),
    postcss({
      minimize: true,
      extract: false,
      modules: true,
    }),
    babel({
      presets: ['@babel/preset-react'],
      extensions: ['.js', '.jsx'],
      plugins: [
        '@babel/plugin-proposal-object-rest-spread',
        '@babel/plugin-proposal-optional-chaining',
        '@babel/plugin-syntax-dynamic-import',
        '@babel/plugin-proposal-class-properties',
        'transform-react-remove-prop-types',
      ],
      exclude: 'node_modules/**',
      babelHelpers: 'bundled',
    }),
    commonjs(),
    copy({
      targets: [
        { src: 'src-js/index.html', dest: 'src-js/dist' },
      ]
    })
  ]
};
