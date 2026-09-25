import type {TreeLinkDatum} from "react-d3-tree"

type TreeNode = TreeLinkDatum["source"]
type Extent = [top: number, bottom: number]

// D3 centers a parent between its outer children, so with uneven subtrees it lands just off a
// child's row and its trunk can run behind a sibling. Re-pack instead: each parent sits on its
// middle child (or midway between the middle pair) and its trunk counts as occupied space.
export function layoutTraceTree(
  root: TreeNode,
  gap: {readonly siblings: number; readonly nonSiblings: number},
): void {
  const pack = (node: TreeNode): Extent[] => {
    const children = node.children ?? []
    const contour: Extent[] = []
    for (const child of children) {
      const childContour = pack(child)
      const overlaps = contour
        .slice(0, childContour.length)
        .map(
          ([, bottom], depth) =>
            bottom + (depth === 0 ? gap.siblings : gap.nonSiblings) - childContour[depth][0],
        )
      const shift = overlaps.length > 0 ? Math.max(...overlaps) : 0
      child.each(descendant => {
        descendant.x += shift
      })
      for (const [depth, [top, bottom]] of childContour.entries()) {
        contour[depth] = [contour[depth]?.[0] ?? top + shift, bottom + shift]
      }
    }
    const first = children.at(0)
    const last = children.at(-1)
    if (!first || !last) return [[node.x, node.x]]
    node.x = (children[(children.length - 1) >> 1].x + children[children.length >> 1].x) / 2
    return [[first.x, last.x], ...contour]
  }

  pack(root)
  const offset = root.x
  root.each(node => {
    node.x -= offset
  })
}
