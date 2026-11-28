// Offline checks that need only Node.js: node tests/frontend-checks.mjs
import fs from 'node:fs';
import assert from 'node:assert/strict';
import vm from 'node:vm';

const html = fs.readFileSync('frontend/index.html', 'utf8');
const js = fs.readFileSync('frontend/app.js', 'utf8');
const conf = JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json', 'utf8'));

// 1) no duplicate ids
const ids = [...html.matchAll(/\sid="([^"]+)"/g)].map(m => m[1]);
const dup = ids.filter((x, i) => ids.indexOf(x) !== i);
assert.deepEqual(dup, [], 'duplicate ids: ' + dup.join(','));

// 2) no remote origins in shipped frontend
for (const [name, text] of [['index.html', html], ['app.js', js]]) {
  assert.ok(!/fonts\.(googleapis|gstatic)\.com/.test(text), name + ' references Google Fonts');
}

// 3) CSP is strict: no unsafe-inline for scripts, no remote script/style sources
const csp = conf.app.security.csp;
assert.ok(/script-src 'self'(;|$)/.test(csp) && !/script-src[^;]*unsafe-inline/.test(csp));
// Remote script/style hosts are forbidden. connect-src may allow https:/http: for subscription fetch only.
assert.ok(!/script-src[^;]*https?:/.test(csp), 'script-src must not allow remote http(s)');
assert.ok(!/style-src[^;]*https?:/.test(csp), 'style-src must not allow remote http(s)');
assert.ok(/connect-src[^;]*ipc:/.test(csp), 'connect-src must keep ipc');
assert.ok(!/<script>[^<]/.test(html) && !/<style>/.test(html), 'inline script/style block present');

// 4) TUN and DNS are available only when native runtime capabilities permit them;
// Kill Switch remains disabled until a real Windows Filtering Platform implementation exists.
for (const id of ['toggle-tun', 'toggle-dns'])
  assert.ok(!new RegExp(`id="${id}"[^>]*disabled`).test(html), id + ' should be capability-gated by the native runtime');
assert.ok(/id="toggle-ks"[^>]*disabled/.test(html), 'Kill Switch must remain disabled');
assert.ok(js.includes('function setSafetyCapabilities') && js.includes('dnsGuard: state.dnsGuard'), 'TUN/DNS capability and invoke wiring missing');
const safetyStart = js.indexOf('function setSafetyCapabilities');
const safetyEnd = js.indexOf('async function emergencyNetworkReset', safetyStart);
const safetySource = js.slice(safetyStart, safetyEnd);
assert.ok(!/state\.core\s*=\s*['\"]sing-box['\"]/.test(safetySource), 'TUN/DNS must not silently switch the selected core');
assert.ok(/kill_switch:\s*false/.test(fs.readFileSync('src-tauri/src/lib.rs', 'utf8')), 'Kill Switch must remain capability-disabled until release gates pass');
assert.ok(js.includes('trafficPollGeneration') && js.includes('trafficUnavailable'), 'traffic polling must handle stale/transient API responses');

// 5) redactSecrets masks credentials
{
  const src = js.slice(js.indexOf('function redactSecrets'), js.indexOf('function addLog'));
  const ctx = {}; vm.createContext(ctx); vm.runInContext(src + ';this.f=redactSecrets', ctx);
  const out = ctx.f('vless://11111111-1111-1111-1111-111111111111@h.test:443?pbk=ABC and https://x.test/sub?token=zzz password=hunter2');
  assert.ok(!/11111111|ABC|zzz|hunter2/.test(out), out);
}

// 6) Complete 9-language i18n coverage & zero language leakage
{
  const i18nStart = js.indexOf('const RTL_LANGS');
  const i18nEnd = js.indexOf('function t(key');
  const i18nSrc = js.slice(i18nStart, i18nEnd);
  const ctx = {};
  vm.createContext(ctx);
  vm.runInContext(i18nSrc + ';this.I18N=I18N;', ctx);
  const langs = ['en', 'zh', 'ko', 'fa', 'ja', 'fr', 'it', 'ar', 'tr'];
  for (const lang of langs) {
    assert.ok(ctx.I18N[lang], `Missing language ${lang}`);
  }
  const enKeys = Object.keys(ctx.I18N.en);
  for (const lang of langs) {
    for (const k of enKeys) {
      assert.ok(typeof ctx.I18N[lang][k] === 'string' && ctx.I18N[lang][k].length > 0, `Missing or empty key ${lang}.${k}`);
    }
    if (lang !== 'fa' && lang !== 'ar') {
      for (const k of enKeys) {
        assert.ok(!/[\u0600-\u06FF]/.test(ctx.I18N[lang][k]), `Persian/Arabic characters leaked into ${lang}.${k}`);
      }
    }
  }
  // Every data-i18n* key in index.html must exist in I18N.en
  const htmlKeys = [
    ...html.matchAll(/data-i18n(?:-placeholder|-title)?="([^"]+)"/g)
  ].map(m => m[1]);
  for (const hk of htmlKeys) {
    assert.ok(enKeys.includes(hk), `HTML key ${hk} missing from I18N.en`);
  }
}

// 7) URI / VMess / Base64 subscription parser checks
{
  const parserStart = js.indexOf('function safeDecodeURIComponent');
  const parserEnd = js.indexOf('// ---------- 6. Render UI');
  const parserSrc = js.slice(parserStart, parserEnd);
  const ctx = {
    state: { activeSub: 'sub-all', servers: [] },
    TextDecoder,
    atob: (s) => Buffer.from(s, 'base64').toString('binary')
  };
  vm.createContext(ctx);
  vm.runInContext(parserSrc + ';this.detectAndParseEntries=detectAndParseEntries;', ctx);

  // Malformed % in fragment must not throw
  const badPct = ctx.detectAndParseEntries('vless://11111111-1111-1111-1111-111111111111@host.test:443#100%bad');
  assert.equal(badPct.length, 1);
  assert.equal(badPct[0].addr, 'host.test:443');

  // VMess JSON base64 decoding
  const vmJson = Buffer.from(JSON.stringify({ v: '2', ps: 'Tokyo-VMess', add: 'vm.example.com', port: '443', id: '1111' })).toString('base64');
  const vmRes = ctx.detectAndParseEntries('vmess://' + vmJson);
  assert.equal(vmRes[0].name, 'Tokyo-VMess');
  assert.equal(vmRes[0].addr, 'vm.example.com:443');

  // Base64 subscription payload
  const subB64 = Buffer.from('trojan://pw@t1.test:443#T1\nvless://id@v1.test:443#V1').toString('base64');
  const subRes = ctx.detectAndParseEntries(subB64);
  assert.equal(subRes.length, 2);
  assert.equal(subRes[0].proto, 'Trojan');
  assert.equal(subRes[1].proto, 'VLESS');
}

console.log('frontend checks PASS');
