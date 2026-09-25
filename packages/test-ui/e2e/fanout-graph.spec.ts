import type {Page} from "@playwright/test"
import {Cell, loadMessage} from "@ton/core"
import type {Trace} from "../src/types/test"

import {
  expect,
  fanoutGraphVisualScenarios,
  stabilizeVisualSnapshot,
  test,
} from "./support/acton-test-ui"
import process from "node:process"

const visualSnapshotsEnabled =
  process.platform === "darwin" && Boolean(process.env.CHECK_UI_SNAPSHOTS)

const waitForNextFrame = async (page: Page) => {
  await page.evaluate(async () => {
    await new Promise<void>(resolve => {
      requestAnimationFrame(() => resolve())
    })
  })
}

const escapeRegExp = (value: string) => value.replaceAll(/[.*+?^${}()|[\]\\]/g, String.raw`\$&`)

const measureGraphContent = async (page: Page): Promise<{height: number; width: number}> => {
  return await page.getByTestId("test-details-content").evaluate(element => {
    element.scrollTo({top: 0, left: 0})

    const contentRect = element.getBoundingClientRect()
    const graph = element.querySelector<HTMLElement>(".rd3t-tree-container")
    const treeContainer = graph?.closest("[class*='treeContainer']") as HTMLElement | null
    const treeRect = treeContainer?.getBoundingClientRect()
    const horizontalContent =
      (treeContainer?.scrollWidth ?? graph?.scrollWidth ?? 0) +
      (treeRect?.left ?? contentRect.left) +
      16

    return {
      height: Math.ceil(contentRect.top + element.scrollHeight + 16),
      width: Math.ceil(
        Math.max(
          document.documentElement.scrollWidth,
          document.body.scrollWidth,
          horizontalContent,
        ),
      ),
    }
  })
}

const fitViewportToGraphContent = async (page: Page) => {
  const originalViewport = page.viewportSize()
  if (!originalViewport) {
    return
  }

  let currentViewport = originalViewport
  for (let i = 0; i < 2; i += 1) {
    const target = await measureGraphContent(page)
    const nextViewport = {
      width: Math.max(currentViewport.width, target.width),
      height: Math.max(currentViewport.height, target.height),
    }

    if (
      nextViewport.width === currentViewport.width &&
      nextViewport.height === currentViewport.height
    ) {
      return
    }

    await page.setViewportSize(nextViewport)
    await waitForNextFrame(page)
    currentViewport = nextViewport
  }
}

const stabilizeFanoutTransactionDetails = async (page: Page) => {
  await page.evaluate(() => {
    const feeValueTitles = new Set(["End Balance", "Total Fee", "Action Fee", "Forward Fee"])
    const rows = [
      ...document.querySelectorAll<HTMLElement>(
        "[class*='labeledSectionRow'], [class*='detailRow']",
      ),
    ]
    const feesRow = rows.find(row => {
      const title = row.querySelector<HTMLElement>(
        "[class*='labeledSectionTitle'], [class*='detailLabel']",
      )
      return title?.textContent?.trim() === "Fees & Sent"
    })

    if (!feesRow) {
      return
    }

    for (const item of feesRow.querySelectorAll<HTMLElement>("[class*='multiColumnItem']")) {
      const title = item.querySelector<HTMLElement>("[class*='multiColumnItemTitle']")
      if (!feeValueTitles.has(title?.textContent?.trim() ?? "")) {
        continue
      }

      item
        .querySelector<HTMLElement>("[class*='multiColumnItemValue']")
        ?.replaceChildren(document.createTextNode("<amount>"))
    }
  })
}

const expectStableGraphScreenshot = async (page: Page, name: string) => {
  const originalViewport = page.viewportSize()

  try {
    await fitViewportToGraphContent(page)
    await stabilizeVisualSnapshot(page)
    await stabilizeFanoutTransactionDetails(page)
    await expect(page).toHaveScreenshot(name, {
      animations: "disabled",
      caret: "hide",
      fullPage: true,
      maxDiffPixels: 200,
    })
  } finally {
    if (originalViewport) {
      await page.setViewportSize(originalViewport)
      await waitForNextFrame(page)
    }
  }
}

const openFanoutGraphScenario = async (
  page: Page,
  scenario: {readonly testName: string; readonly traceName: string},
) => {
  await page.getByRole("button", {name: new RegExp(escapeRegExp(scenario.testName))}).click()
  await expect(page.getByTestId("test-details-title")).toContainText(scenario.testName)

  const transactionsTab = page.getByRole("tab", {name: "Transactions"})
  await transactionsTab.click()
  await expect(transactionsTab).toHaveAttribute("aria-selected", "true")

  const traceTab = page.getByRole("button", {name: scenario.traceName, exact: true})
  await expect(traceTab).toBeVisible()
  await traceTab.click()
  await expect(traceTab).toHaveAttribute("aria-current", "true")

  await expect(page.locator(".rd3t-tree-container svg").first()).toBeVisible()
  const firstTransaction = page.getByRole("button", {name: /^Transaction /}).first()
  await expect(firstTransaction).toBeVisible()
  await firstTransaction.click()
  await expect(page.getByText("Message route", {exact: true})).toBeVisible()
}

test("opens and scrolls to the fifth transaction logs in a twenty-transaction trace", async ({
  loggedFanoutGraphUi,
  page,
}) => {
  const traceResponse = page.waitForResponse("**/api/trace/**")
  await page.goto(loggedFanoutGraphUi.baseUrl)
  await openFanoutGraphScenario(page, {
    testName: "chain graph has twenty transactions",
    traceName: "chain 20",
  })
  const trace = (await (await traceResponse).json()) as Trace
  const transactions = trace.traces.find(entry => entry.name === "chain 20")?.transactions ?? []
  expect(transactions).toHaveLength(20)

  const fifth = transactions[4]
  const id = Cell.fromBase64(fifth.raw_transaction).hash().toString("hex")
  await page.getByRole("button", {name: `Transaction ${id}`, exact: true}).click()
  await page.getByRole("button", {name: "View logs", exact: true}).click()

  await expect(page.getByRole("tab", {name: "Logs", exact: true})).toHaveAttribute(
    "aria-selected",
    "true",
  )
  await expect(page.getByRole("region", {name: /^Transaction #\d+ logs$/})).toHaveCount(20)
  const logs = page.getByRole("region", {name: "Transaction #5 logs", exact: true})
  await expect(logs).toBeFocused()
  await expect(logs.getByText("Transaction #5", {exact: true})).toBeInViewport()

  const content = page.getByTestId("test-details-content")
  expect(await content.evaluate(element => element.scrollTop)).toBeGreaterThan(0)
  expect((await logs.boundingBox())?.y).toBeCloseTo((await content.boundingBox())?.y ?? -1, 0)
  await expect(logs.getByRole("button", {name: "Collapse VM log", exact: true})).toBeVisible()
  await expect(logs.locator("pre")).toHaveText(fifth.vm_log_diff)
})

test.describe("Fanout graph visual snapshots", () => {
  test.skip(
    !visualSnapshotsEnabled,
    "Set CHECK_UI_SNAPSHOTS to run fanout graph visual snapshot checks on macOS",
  )

  for (const scenario of fanoutGraphVisualScenarios) {
    test(`matches ${scenario.traceName}`, async ({fanoutGraphUi, page}) => {
      await page.goto(fanoutGraphUi.baseUrl)

      await openFanoutGraphScenario(page, scenario)
      await expectStableGraphScreenshot(page, scenario.snapshotName)
    })
  }
})

test("rounds only the outer branches of a shared fanout spine", async ({fanoutGraphUi, page}) => {
  await page.goto(fanoutGraphUi.baseUrl)
  await openFanoutGraphScenario(page, {
    testName: "wide fanout graph has six outgoing messages",
    traceName: "wide fanout 6",
  })

  const links = page.locator(".rd3t-tree-container path.rd3t-link")
  await expect(links).toHaveCount(7)
  const branches = await links.evaluateAll(elements =>
    elements
      .map(element => {
        const path = element as SVGPathElement
        return {
          path: path.getAttribute("d") ?? "",
          startY: path.getPointAtLength(0).y,
          endY: path.getPointAtLength(path.getTotalLength()).y,
        }
      })
      .filter(link => link.startY !== link.endY)
      .sort((left, right) => left.endY - right.endY),
  )
  expect(branches).toHaveLength(6)
  expect(branches[0].path).toMatch(/[Aa]/)
  expect(branches.at(-1)?.path).toMatch(/[Aa]/)
  for (const branch of branches.slice(1, -1)) {
    expect(branch.path).toMatch(/^M[^A-Za-z]+V[^A-Za-z]+H[^A-Za-z]+$/)
  }
})

test("aligns the middle branch with its parent when two chains precede a leaf", async ({
  fanoutGraphUi,
  page,
}) => {
  await page.goto(fanoutGraphUi.baseUrl)
  await openFanoutGraphScenario(page, {
    testName: "three branches mix two chains and one leaf",
    traceName: "two chains and one leaf",
  })

  const tree = page.locator(".rd3t-tree-container")
  await expect(tree.locator('circle[aria-label^="Transaction "]')).toHaveCount(6)
  const {nodes, links} = await tree.evaluate(element => ({
    nodes: [...element.querySelectorAll('circle[aria-label^="Transaction "]')].map(circle => {
      const bounds = circle.getBoundingClientRect()
      return {x: bounds.x + bounds.width / 2, y: bounds.y + bounds.height / 2}
    }),
    links: [...element.querySelectorAll<SVGPathElement>("path.rd3t-link")].map(path => {
      const matrix = path.getScreenCTM()
      const start = path.getPointAtLength(0).matrixTransform(matrix ?? undefined)
      const end = path.getPointAtLength(path.getTotalLength()).matrixTransform(matrix ?? undefined)
      return {path: path.getAttribute("d") ?? "", startX: start.x, endY: end.y}
    }),
  }))
  const columns = [...new Set(nodes.map(node => node.x))].sort((left, right) => left - right)
  expect(columns).toHaveLength(3)
  const parent = nodes.find(node => node.x === columns[0])
  expect(parent).toBeDefined()
  const children = nodes
    .filter(node => node.x === columns[1])
    .sort((left, right) => left.y - right.y)
  expect(children).toHaveLength(3)
  expect(children[1].y).toBeCloseTo(parent?.y ?? Number.NaN, 1)

  const branches = links
    .filter(link => Math.abs(link.startX - columns[0]) < 0.5)
    .sort((left, right) => left.endY - right.endY)
  expect(branches).toHaveLength(3)
  expect(branches[0].path).toMatch(/[Aa]/)
  expect(branches[1].path).not.toMatch(/[Aa]/)
  expect(branches[2].path).toMatch(/[Aa]/)
})

test("external-out graph node opens its message details", async ({fanoutGraphUi, page}) => {
  await page.goto(fanoutGraphUi.baseUrl)

  const scenario = fanoutGraphVisualScenarios.find(
    item => item.testName === "graph has internal and external out children",
  )
  if (!scenario) {
    throw new Error("External-out fanout scenario is not configured")
  }

  await openFanoutGraphScenario(page, scenario)

  const selectedNode = page.locator('circle[aria-label^="Transaction "][aria-pressed="true"]')
  const externalOutNode = page.getByRole("button", {
    name: /^External-out message \d+ from transaction /,
  })

  await expect(externalOutNode).toBeVisible()
  const externalOutLabel = await externalOutNode.getAttribute("aria-label")
  const parentTransactionId = externalOutLabel?.split(" from transaction ")[1]
  if (!parentTransactionId) {
    throw new Error("External-out node does not identify its parent transaction")
  }
  await externalOutNode.click()

  const details = page.getByRole("region", {name: "External-out message", exact: true})
  await expect(details).toBeVisible()
  await expect(details).toContainText("ExternalLogNotice")
  await expect(externalOutNode).toHaveAttribute("aria-pressed", "true")
  await expect(selectedNode).toHaveCount(0)

  await page.getByRole("button", {name: `Transaction ${parentTransactionId}`, exact: true}).click()
  await expect(details).toHaveCount(0)
  await expect(selectedNode).toHaveAttribute("aria-label", `Transaction ${parentTransactionId}`)
  await expect(externalOutNode).toHaveAttribute("aria-pressed", "false")

  await externalOutNode.focus()
  await page.keyboard.press("Enter")
  await expect(details).toBeVisible()
  await expect(externalOutNode).toHaveAttribute("aria-pressed", "true")

  await page.keyboard.press("Space")
  await expect(details).toHaveCount(0)
  await expect(externalOutNode).toHaveAttribute("aria-pressed", "false")
})

test("each external-out node opens its own body", async ({fanoutGraphUi, page}) => {
  await page.goto(fanoutGraphUi.baseUrl)
  await openFanoutGraphScenario(page, {
    testName: "inspect external-out messages",
    traceName: "Two external-out messages",
  })

  const externalOutNodes = page.getByRole("button", {name: /^External-out message /})
  await expect(externalOutNodes).toHaveCount(2)
  const details = page.getByRole("region", {name: "External-out message", exact: true})

  await externalOutNodes.nth(0).click()
  await expect(details).toContainText("PaymentSent")
  await expect(details).toContainText("0x71000002")
  await expect(details).toContainText("1250000000")
  await expect(details.getByRole("button", {name: "Copy raw body", exact: true})).toBeVisible()

  await page.context().grantPermissions(["clipboard-read", "clipboard-write"])
  await details.getByText("Message data", {exact: true}).locator("..").hover()
  await details.getByRole("button", {name: "Copy raw body", exact: true}).click()
  const bodyBoc = await page.evaluate(() => navigator.clipboard.readText())
  await details.getByRole("button", {name: "Copy raw message", exact: true}).click()
  const messageBoc = await page.evaluate(() => navigator.clipboard.readText())
  const body = Cell.fromBoc(Buffer.from(bodyBoc, "hex"))[0]
  const messageCell = Cell.fromBoc(Buffer.from(messageBoc, "hex"))[0]
  const message = loadMessage(messageCell.beginParse())
  expect(message.info.type).toBe("external-out")
  expect(message.body.equals(body)).toBe(true)
  expect(body.beginParse().loadUint(32)).toBe(0x71_00_00_02)

  // The second log has dictionary key 2 because an internal message occupies key 1.
  await expect(externalOutNodes.nth(1)).toHaveAttribute("aria-label", /^External-out message 2 /)
  await externalOutNodes.nth(1).click()
  await expect(details.getByText("unknown", {exact: true})).toBeVisible()
  await expect(details).toContainText("Cell(")
  await expect(details).toContainText("External<16:48879>")
  await expect(details).not.toContainText("PaymentSent")
  await expect(externalOutNodes.nth(0)).toHaveAttribute("aria-pressed", "false")
  await expect(externalOutNodes.nth(1)).toHaveAttribute("aria-pressed", "true")

  await externalOutNodes.nth(0).click()
  await expect(details).toContainText("PaymentSent")
  await expect(details.getByText("unknown", {exact: true})).toHaveCount(0)
})
