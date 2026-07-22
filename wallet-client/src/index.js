/**
 * MilestoneX Wallet Client — index.js
 *
 * Implements:
 *  - Freighter wallet connect/disconnect lifecycle
 *  - Campaign state display with refresh
 *  - Multi-asset donation flow (XLM + Stellar assets)
 *  - XDR signing via Freighter and submission via Horizon
 *  - Campaign ID via URL query param (?campaign=<CONTRACT_ID>)
 *
 * Dependencies (bundled):
 *   @stellar/freighter-api ^3
 *   @stellar/stellar-sdk  ^13
 */

import {
  isConnected,
  isAllowed,
  requestAccess,
  getAddress,
  getNetwork,
  signTransaction,
} from "@stellar/freighter-api";

import {
  Networks,
  Server,
  TransactionBuilder,
  BASE_FEE,
  Asset,
  Operation,
  Keypair,
  Contract,
  xdr,
  nativeToScVal,
  scValToNative,
  Address,
  SorobanRpc,
} from "@stellar/stellar-sdk";

// ─── Constants ────────────────────────────────────────────────────────────────

const NETWORK_CONFIG = {
  testnet: {
    passphrase: Networks.TESTNET,
    horizonUrl: "https://horizon-testnet.stellar.org",
    rpcUrl: "https://soroban-testnet.stellar.org",
    explorerBase: "https://stellar.expert/explorer/testnet/tx",
  },
  mainnet: {
    passphrase: Networks.PUBLIC,
    horizonUrl: "https://horizon.stellar.org",
    rpcUrl: "https://soroban-mainnet.stellar.org", // community RPC — replace for prod
    explorerBase: "https://stellar.expert/explorer/public/tx",
  },
};

// ─── State ───────────────────────────────────────────────────────────────────

const state = {
  walletAddress: null,
  network: "testnet",            // resolved from Freighter on connect
  campaignId: null,              // Soroban contract ID
  campaignData: null,            // last fetched campaign report
  acceptedAssets: [],            // [{type:"native"} | {type:"stellar",code,issuer}]
  pendingXdr: null,              // signed XDR waiting for submission
};

// ─── DOM helpers ─────────────────────────────────────────────────────────────

const $ = (id) => document.getElementById(id);

function setStatus(msg, level = "") {
  const bar = $("status-bar");
  bar.className = level ? `${level}` : "";
  bar.innerHTML = msg;
}

function spin(msg) {
  setStatus(`<span class="spinner"></span> ${msg}`, "info");
}

// ─── Wallet ───────────────────────────────────────────────────────────────────

async function connectWallet() {
  const btn = $("connect-btn");
  btn.disabled = true;
  spin("Connecting to Freighter…");

  try {
    // Check extension presence
    const connected = await isConnected();
    if (!connected.isConnected) {
      throw new Error(
        "Freighter extension not detected. Install it from freighter.app and reload."
      );
    }

    // Request access (shows Freighter popup if not yet allowed)
    const allowed = await isAllowed();
    if (!allowed.isAllowed) {
      const accessResult = await requestAccess();
      if (accessResult.error) {
        throw new Error(`Freighter access denied: ${accessResult.error}`);
      }
    }

    // Get address
    const addrResult = await getAddress();
    if (addrResult.error) {
      throw new Error(`Could not get address: ${addrResult.error}`);
    }

    // Get network
    const netResult = await getNetwork();
    const detectedNetwork =
      netResult.network === "PUBLIC" ? "mainnet" : "testnet";

    state.walletAddress = addrResult.address;
    state.network = detectedNetwork;

    // Update badge
    $("network-badge").textContent = detectedNetwork;

    showConnected(addrResult.address);
    setStatus(`Connected as ${truncate(addrResult.address)} on ${detectedNetwork}`, "ok");
  } catch (err) {
    setStatus(`❌ ${err.message}`, "err");
    btn.disabled = false;
  }
}

function showConnected(address) {
  const dot = $("conn-dot");
  dot.classList.add("connected");
  $("wallet-address-display").textContent = address;

  const btn = $("connect-btn");
  btn.textContent = "Disconnect";
  btn.classList.add("secondary");
  btn.onclick = disconnectWallet;
  btn.disabled = false;

  // Enable donate button if a campaign is already loaded
  if (state.campaignId) {
    $("donate-btn").disabled = false;
  }
}

function disconnectWallet() {
  state.walletAddress = null;

  $("conn-dot").classList.remove("connected");
  $("wallet-address-display").textContent = "Not connected";

  const btn = $("connect-btn");
  btn.textContent = "Connect Freighter";
  btn.classList.remove("secondary");
  btn.onclick = connectWallet;
  btn.disabled = false;

  $("donate-btn").disabled = true;
  setStatus("Wallet disconnected.", "warn");
}

// ─── Campaign loading ─────────────────────────────────────────────────────────

async function loadCampaign() {
  const input = $("campaign-id-input").value.trim();
  if (!input) {
    setStatus("Enter a contract ID first.", "warn");
    return;
  }

  state.campaignId = input;

  // Persist into URL query param so a reload returns the same campaign
  const url = new URL(window.location.href);
  url.searchParams.set("campaign", input);
  window.history.replaceState({}, "", url.toString());

  await refreshCampaign();
}

async function refreshCampaign() {
  if (!state.campaignId) return;

  spin("Fetching campaign state…");
  $("refresh-btn").disabled = true;

  try {
    const data = await callContractReadOnly("get_campaign_report", []);
    const status = await callContractReadOnly("get_campaign_status", []);
    const milestones = await callAllMilestones();
    const assetsSummary = await fetchAcceptedAssets();

    state.campaignData = { data, status, milestones, assetsSummary };
    renderCampaign(state.campaignData);

    setStatus("Campaign loaded.", "ok");
  } catch (err) {
    setStatus(`❌ ${err.message}`, "err");
  } finally {
    $("refresh-btn").disabled = false;
  }
}

// ─── Soroban RPC helpers ──────────────────────────────────────────────────────

function getServer() {
  const cfg = NETWORK_CONFIG[state.network];
  return new SorobanRpc.Server(cfg.rpcUrl, { allowHttp: false });
}

/**
 * Call a read-only Soroban contract method using simulateTransaction.
 * Returns the decoded native JS value of the first return value.
 */
async function callContractReadOnly(method, args) {
  const server = getServer();
  const cfg = NETWORK_CONFIG[state.network];

  // We need a source account for simulation; use a well-known testnet pubkey
  // when the wallet is not connected (read-only path).
  const source = state.walletAddress || "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN";

  const account = await server.getAccount(source);
  const contract = new Contract(state.campaignId);

  const tx = new TransactionBuilder(account, {
    fee: BASE_FEE,
    networkPassphrase: cfg.passphrase,
  })
    .addOperation(contract.call(method, ...args))
    .setTimeout(30)
    .build();

  const simResult = await server.simulateTransaction(tx);

  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation error: ${simResult.error}`);
  }

  if (!simResult.result) {
    return null;
  }

  const retVal = simResult.result.retval;
  return scValToNative(retVal);
}

/**
 * Fetch all milestones from the contract. The contract exposes
 * `get_all_milestones` which returns a Vec of MilestoneView.
 */
async function callAllMilestones() {
  try {
    const result = await callContractReadOnly("get_all_milestones", []);
    return Array.isArray(result) ? result : [];
  } catch (_) {
    return [];
  }
}

/**
 * Derive accepted assets from the campaign report if available.
 */
async function fetchAcceptedAssets() {
  // campaign report contains accepted_assets via get_campaign_report
  // We parse them from the already-fetched campaignData or re-fetch
  try {
    const report = await callContractReadOnly("get_campaign_report", []);
    if (report && report.accepted_assets) {
      return report.accepted_assets;
    }
  } catch (_) {}
  return [];
}

// ─── Render campaign state ────────────────────────────────────────────────────

function renderCampaign({ data, status, milestones, assetsSummary }) {
  $("campaign-state-card").style.display = "";
  $("donate-card").style.display = "";

  // Title (use campaign ID as fallback)
  const titleEl = $("campaign-title");
  titleEl.textContent = truncate(state.campaignId, 20);

  // Status pill
  const pillEl = $("campaign-status-pill");
  const statusStr = resolveStatus(status);
  pillEl.textContent = statusStr;
  pillEl.className = `status-pill ${statusStr.toLowerCase().replace(/\s/g, "")}`;

  // Stats grid
  const totalRaised = data ? BigInt(data.total_raised ?? data.raised_amount ?? 0) : 0n;
  const goalAmount  = data ? BigInt(data.goal_amount ?? 0) : 0n;
  const donorCount  = data ? (data.donor_count ?? data.unique_donor_count ?? 0) : 0;
  const donationCount = data ? (data.donation_count ?? 0) : 0;
  const daysRemaining = status ? (status.days_remaining ?? "—") : "—";

  const stats = [
    { label: "Raised",      value: formatStroops(totalRaised) },
    { label: "Goal",        value: formatStroops(goalAmount) },
    { label: "Donors",      value: donorCount },
    { label: "Donations",   value: donationCount },
    { label: "Days Left",   value: daysRemaining },
  ];

  const grid = $("stat-grid");
  grid.innerHTML = stats
    .map(
      (s) =>
        `<div class="stat-box"><div class="label">${s.label}</div><div class="value">${s.value}</div></div>`
    )
    .join("");

  // Progress
  const pct =
    goalAmount > 0n
      ? Math.min(100, Number((totalRaised * 10000n) / goalAmount) / 100)
      : 0;
  $("progress-fill").style.width = `${pct.toFixed(1)}%`;
  $("progress-label").textContent = `${pct.toFixed(1)}% funded (${formatStroops(totalRaised)} / ${formatStroops(goalAmount)})`;

  // Milestones
  const milestoneList = $("milestone-list");
  if (!milestones || milestones.length === 0) {
    milestoneList.innerHTML = `<li style="color:var(--muted);font-size:.85rem">No milestones loaded.</li>`;
  } else {
    milestoneList.innerHTML = milestones
      .map((m, i) => renderMilestone(m, i))
      .join("");
  }

  // Populate asset select
  populateAssetSelect(assetsSummary);

  // Enable donate button if wallet connected
  if (state.walletAddress) {
    $("donate-btn").disabled = false;
  }
}

function renderMilestone(m, index) {
  // m may be a raw scVal-decoded object or already parsed
  const mData = m.data ?? m;
  const mStatus = (mData.status ?? m.status ?? "locked").toString().toLowerCase();
  const targetAmount = BigInt(mData.target_amount ?? m.target_amount ?? 0);
  const releasedAmount = BigInt(mData.released_amount ?? m.released_amount ?? 0);
  const description = mData.description ?? m.description ?? `Milestone ${index + 1}`;

  const icons = { locked: "🔒", unlocked: "🔓", released: "✅" };
  const icon = icons[mStatus] ?? "📌";
  const statusLabel = mStatus.charAt(0).toUpperCase() + mStatus.slice(1);

  return `
    <li class="milestone-item">
      <span class="m-icon">${icon}</span>
      <span class="m-label">${escapeHtml(String(description))}</span>
      <span class="m-amount">${formatStroops(targetAmount)}</span>
      <span class="m-status ${mStatus}">${statusLabel}</span>
    </li>`;
}

function populateAssetSelect(assets) {
  const sel = $("asset-select");
  // Always include native XLM
  const options = [`<option value="native">XLM (native)</option>`];

  if (Array.isArray(assets)) {
    assets.forEach((a) => {
      if (a.asset_type === "stellar" || a.type === "Stellar" || a.asset_code) {
        const code = a.asset_code ?? a.code ?? "UNKNOWN";
        const issuer = a.issuer ?? a.issuer_address ?? "";
        const val = JSON.stringify({ type: "stellar", code, issuer });
        options.push(
          `<option value='${escapeAttr(val)}'>${escapeHtml(code)} (${truncate(issuer, 8)})</option>`
        );
      }
    });
  }

  state.acceptedAssets = assets;
  sel.innerHTML = options.join("");
}

// ─── Donation flow ─────────────────────────────────────────────────────────────

async function startDonation() {
  if (!state.walletAddress) {
    setStatus("Connect your wallet first.", "warn");
    return;
  }
  if (!state.campaignId) {
    setStatus("Load a campaign first.", "warn");
    return;
  }

  const amountStr = $("donate-amount").value.trim();
  const amount = parseInt(amountStr, 10);
  if (!amount || amount <= 0) {
    setStatus("Enter a valid amount in stroops.", "warn");
    return;
  }

  const assetValue = $("asset-select").value;
  const memo = $("memo-input").value.trim();

  $("donate-btn").disabled = true;
  spin("Building donation transaction…");

  try {
    // Build the Soroban invocation transaction
    const signedXdr = await buildAndSignDonation(amount, assetValue, memo);
    state.pendingXdr = signedXdr;

    // Show XDR panel
    $("xdr-text").value = signedXdr;
    $("xdr-panel").style.display = "";
    document.getElementById("xdr-panel").scrollIntoView({ behavior: "smooth" });

    setStatus("Transaction signed. Review the XDR and click Submit to broadcast.", "info");
  } catch (err) {
    setStatus(`❌ ${err.message}`, "err");
  } finally {
    $("donate-btn").disabled = false;
  }
}

async function buildAndSignDonation(amountStroops, assetValue, memo) {
  const server = getServer();
  const cfg = NETWORK_CONFIG[state.network];

  const sourceAccount = await server.getAccount(state.walletAddress);
  const contract = new Contract(state.campaignId);

  // Build AssetInfo ScVal for the contract's donate(donor, amount, asset) call
  const assetScVal = buildAssetInfoScVal(assetValue, cfg.passphrase);

  const donorAddress = new Address(state.walletAddress).toScVal();
  const amountScVal = nativeToScVal(BigInt(amountStroops), { type: "i128" });

  let txBuilder = new TransactionBuilder(sourceAccount, {
    fee: String(BASE_FEE * 10), // slightly higher fee for contract invocations
    networkPassphrase: cfg.passphrase,
  }).addOperation(contract.call("donate", donorAddress, amountScVal, assetScVal));

  if (memo) {
    txBuilder = txBuilder.addMemo(
      // Use text memo; Soroban contract doesn't use the memo but it's visible in explorer
      { type: "text", value: memo.slice(0, 28) }
    );
  }

  txBuilder = txBuilder.setTimeout(180);
  const tx = txBuilder.build();

  // Simulate first to get footprint + auth
  const simResult = await server.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation failed: ${simResult.error}`);
  }

  // Assemble the transaction with the simulation footprint
  const assembledTx = SorobanRpc.assembleTransaction(tx, simResult).build();

  // Sign via Freighter
  const signedResult = await signTransaction(assembledTx.toXDR(), {
    networkPassphrase: cfg.passphrase,
    address: state.walletAddress,
  });

  if (signedResult.error) {
    throw new Error(`Signing failed: ${signedResult.error}`);
  }

  return signedResult.signedTxXdr;
}

/**
 * Build a Soroban ScVal that matches the contract's `AssetInfo` type.
 *
 * The `AssetInfo` enum on the contract has two variants:
 *   - Native
 *   - Stellar { asset_code: String, issuer: Address }
 */
function buildAssetInfoScVal(assetValue, networkPassphrase) {
  if (assetValue === "native") {
    // AssetInfo::Native — Soroban enum variant with no fields
    return xdr.ScVal.scvVec([
      xdr.ScVal.scvSymbol("Native"),
    ]);
  }

  let parsed;
  try {
    parsed = JSON.parse(assetValue);
  } catch (_) {
    // Fallback: treat as native
    return xdr.ScVal.scvVec([xdr.ScVal.scvSymbol("Native")]);
  }

  const { code, issuer } = parsed;

  // AssetInfo::Stellar { asset_code, issuer } — encoded as a map
  const assetCodeScVal = xdr.ScVal.scvString(Buffer.from(code, "utf8"));
  const issuerScVal = new Address(issuer).toScVal();

  return xdr.ScVal.scvVec([
    xdr.ScVal.scvSymbol("Stellar"),
    xdr.ScVal.scvMap([
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("asset_code"),
        val: assetCodeScVal,
      }),
      new xdr.ScMapEntry({
        key: xdr.ScVal.scvSymbol("issuer"),
        val: issuerScVal,
      }),
    ]),
  ]);
}

// ─── Submit signed XDR ────────────────────────────────────────────────────────

async function submitXdr() {
  if (!state.pendingXdr) return;

  $("submit-xdr-btn").disabled = true;
  spin("Submitting transaction to the network…");

  try {
    const server = getServer();
    const cfg = NETWORK_CONFIG[state.network];

    // Parse the signed XDR back into a transaction
    const tx = TransactionBuilder.fromXDR(state.pendingXdr, cfg.passphrase);

    const sendResult = await server.sendTransaction(tx);

    if (sendResult.status === "ERROR") {
      throw new Error(`Send error: ${JSON.stringify(sendResult.errorResult)}`);
    }

    const hash = sendResult.hash;

    // Poll for confirmation
    spin("Waiting for confirmation…");
    const confirmed = await pollTransaction(server, hash);

    if (confirmed.status === "SUCCESS") {
      showTxResult(hash, cfg.explorerBase);
      setStatus("✅ Donation submitted successfully!", "ok");
      $("xdr-panel").style.display = "none";
      state.pendingXdr = null;
      // Auto-refresh campaign state
      await refreshCampaign();
    } else {
      throw new Error(
        `Transaction failed: ${confirmed.resultXdr ?? confirmed.status}`
      );
    }
  } catch (err) {
    setStatus(`❌ Submission failed: ${err.message}`, "err");
  } finally {
    $("submit-xdr-btn").disabled = false;
  }
}

async function pollTransaction(server, hash, maxAttempts = 20, intervalMs = 2000) {
  for (let i = 0; i < maxAttempts; i++) {
    await delay(intervalMs);
    const result = await server.getTransaction(hash);
    if (result.status !== SorobanRpc.Api.GetTransactionStatus.NOT_FOUND &&
        result.status !== "NOT_FOUND") {
      return result;
    }
  }
  throw new Error("Transaction confirmation timed out. Check the explorer for the final status.");
}

function showTxResult(hash, explorerBase) {
  $("tx-result").style.display = "";
  $("tx-hash").textContent = hash;
  const link = $("explorer-link");
  link.href = `${explorerBase}/${hash}`;
}

// ─── Utility helpers ──────────────────────────────────────────────────────────

/** Format stroops (1/10_000_000 XLM) as "X.XXXXXXX XLM" */
function formatStroops(stroops) {
  const n = typeof stroops === "bigint" ? stroops : BigInt(stroops ?? 0);
  const xlm = Number(n) / 1e7;
  return `${xlm.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 7 })} XLM`;
}

function truncate(str, len = 16) {
  if (!str) return "";
  return str.length > len ? `${str.slice(0, 6)}…${str.slice(-6)}` : str;
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function escapeAttr(s) {
  return String(s).replace(/'/g, "&#39;").replace(/"/g, "&quot;");
}

function delay(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

function resolveStatus(statusObj) {
  if (!statusObj) return "Unknown";
  const raw = statusObj.status ?? statusObj;
  if (typeof raw === "object") {
    // Soroban enum decoded as { Active: null } or similar
    const key = Object.keys(raw)[0];
    return key ?? "Unknown";
  }
  return String(raw);
}

// ─── Boot ─────────────────────────────────────────────────────────────────────

function init() {
  // Wire up buttons
  $("connect-btn").onclick = connectWallet;
  $("load-campaign-btn").onclick = loadCampaign;
  $("refresh-btn").onclick = refreshCampaign;
  $("donate-btn").onclick = startDonation;
  $("submit-xdr-btn").onclick = submitXdr;
  $("copy-xdr-btn").onclick = () => {
    navigator.clipboard
      .writeText($("xdr-text").value)
      .then(() => setStatus("XDR copied to clipboard.", "ok"))
      .catch(() => setStatus("Could not copy — use Ctrl+C in the text area.", "warn"));
  };

  // Allow pressing Enter in the campaign input to load
  $("campaign-id-input").addEventListener("keydown", (e) => {
    if (e.key === "Enter") loadCampaign();
  });

  // Pre-fill campaign ID from query param
  const params = new URLSearchParams(window.location.search);
  const campaignParam = params.get("campaign");
  if (campaignParam) {
    $("campaign-id-input").value = campaignParam;
    state.campaignId = campaignParam;
    // Auto-load after a short tick to let the DOM settle
    setTimeout(refreshCampaign, 100);
  }

  setStatus("Ready. Connect your Freighter wallet to get started.");
}

// Run on DOM ready
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", init);
} else {
  init();
}
