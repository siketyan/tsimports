use biome_js_syntax::JsFileSource;
use insta::assert_snapshot;
use tsimports::tsimports;

#[test]
fn snapshot_tests() {
    assert_snapshot!(tsimports(
        "\
import fs from 'fs'
import path from 'path'
import _ from 'lodash'
import chalk from 'chalk'
import foo from 'src/foo'
import foo from '../foo'
import qux from '../../foo/qux'
import bar from './bar'
import baz from './bar/baz'
import main from './'
import log = console.log
import type { Foo, Bar } from 'foo'
import userEvent from '@testing-library/user-event'
",
        JsFileSource::ts(),
    )
    .unwrap());
}
