import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { useDatasources } from "@/hooks/use-datasources"
import { api } from "@/lib/api"
import { useState } from "react"
import { toast } from "sonner"

const AGGREGATE_FUNCTIONS = [
  "sum",
  "avg",
  "median",
  "min",
  "max",
  "count",
] as const

interface FilterStep {
  type: "filter"
  column: string
  operator: string
  value: string
}

interface AggregateStep {
  type: "aggregate"
  columns: string[]
  functions: string[]
}

interface GroupByStep {
  type: "group_by"
  by: string
  column: string
  function: string
}

type PipelineStep = FilterStep | AggregateStep | GroupByStep

export default function JobForm({
  onJobCreated,
}: {
  onJobCreated?: (datasourceId: string) => void
}) {
  const { datasources } = useDatasources()
  const [name, setName] = useState("")
  const [datasourceId, setDatasourceId] = useState("")
  const [steps, setSteps] = useState<PipelineStep[]>([])
  const [submitting, setSubmitting] = useState(false)

  const [filterOpen, setFilterOpen] = useState(false)
  const [stepColumn, setStepColumn] = useState("")
  const [stepOperator, setStepOperator] = useState("==")
  const [stepValue, setStepValue] = useState("")

  const [aggOpen, setAggOpen] = useState(false)
  const [aggColumns, setAggColumns] = useState("")
  const [aggFunctions, setAggFunctions] = useState<string[]>([])

  const [groupByOpen, setGroupByOpen] = useState(false)
  const [groupByColumn, setGroupByColumn] = useState("")
  const [groupByFunction, setGroupByFunction] = useState("sum")
  const [groupByBy, setGroupByBy] = useState("")

  function addGroupByStep() {
    if (!groupByBy.trim() || !groupByColumn.trim()) return
    const step: GroupByStep = {
      type: "group_by",
      by: groupByBy,
      column: groupByColumn,
      function: groupByFunction,
    }
    setSteps([...steps, step])
    setGroupByBy("")
    setGroupByColumn("")
    setGroupByFunction("sum")
    setGroupByOpen(false)
  }

  function addFilterStep() {
    if (!stepColumn.trim()) return
    const step: FilterStep = {
      type: "filter",
      column: stepColumn,
      operator: stepOperator,
      value: stepValue,
    }
    setSteps([...steps, step])
    setStepColumn("")
    setStepOperator("==")
    setStepValue("")
    setFilterOpen(false)
  }

  function addAggregateStep() {
    const columns = aggColumns
      .split(",")
      .map((c) => c.trim())
      .filter(Boolean)
    if (columns.length === 0) return
    if (aggFunctions.length === 0) return
    const step: AggregateStep = {
      type: "aggregate",
      columns,
      functions: aggFunctions,
    }
    setSteps([...steps, step])
    setAggColumns("")
    setAggFunctions([])
    setAggOpen(false)
  }

  function toggleFunction(fn: string) {
    setAggFunctions((prev) =>
      prev.includes(fn) ? prev.filter((f) => f !== fn) : [...prev, fn],
    )
  }

  function removeStep(index: number) {
    setSteps(steps.filter((_, i) => i !== index))
  }

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    if (!name.trim() || !datasourceId) return

    setSubmitting(true)
    try {
      const pipeline = steps.map((s) => {
        if (s.type === "filter") {
          return {
            op: "filter",
            column: s.column,
            operator: s.operator,
            value: tryParseJson(s.value),
          }
        }
        if (s.type === "group_by") {
          return {
            op: "group_by",
            by: s.by,
            aggregate: { column: s.column, function: s.function },
          }
        }
        return {
          op: "aggregate",
          columns: s.columns,
          functions: s.functions,
        }
      })

      await api.post("/jobs", { name, datasource_id: datasourceId, pipeline })

      toast.success("Job created successfully")
      onJobCreated?.(datasourceId)
      setName("")
      setDatasourceId("")
      setSteps([])
    } catch (err) {
      toast.error(err instanceof Error ? err.message : "Failed to create job")
    } finally {
      setSubmitting(false)
    }
  }

  return (
    <form onSubmit={handleSubmit} className="flex w-full flex-col gap-4 p-4">
      <div className="flex flex-col gap-1.5">
        <Label htmlFor="job-name">Job name</Label>
        <Input
          id="job-name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="My job"
        />
      </div>

      <div className="flex flex-col gap-1.5">
        <Label htmlFor="job-ds">Datasource</Label>
        <select
          id="job-ds"
          value={datasourceId}
          onChange={(e) => setDatasourceId(e.target.value)}
          className="h-8 rounded-lg border border-input bg-white! px-2.5 text-sm"
        >
          <option value="">Select a datasource...</option>
          {datasources.map((ds) => (
            <option key={ds.id} value={ds.id}>
              {ds.name}
            </option>
          ))}
        </select>
      </div>

      <div className="flex flex-col gap-2">
        <div className="flex items-center justify-between">
          <span className="text-sm font-medium text-gray-700">
            Pipeline steps
          </span>
          <div className="flex gap-1">
            <Button
              type="button"
              variant={"outline"}
              className="hover:bg-[#8828ad] hover:text-white border-[#8828ad] text-[#8828ad] shadow-sm transition-all active:scale-[0.98]"
              size="xs"
              onClick={() => setFilterOpen(true)}
            >
              + Filter
            </Button>
            <Button
              type="button"
              variant="outline"
              className="hover:bg-[#8828ad] hover:text-white border-[#8828ad] text-[#8828ad] shadow-sm transition-all active:scale-[0.98]"
              size="xs"
              onClick={() => setAggOpen(true)}
            >
              + Aggregate
            </Button>
            <Button
              type="button"
              variant="outline"
              className="hover:bg-[#8828ad] hover:text-white border-[#8828ad] text-[#8828ad] shadow-sm transition-all active:scale-[0.98]"
              size="xs"
              onClick={() => setGroupByOpen(true)}
            >
              + Group By
            </Button>
          </div>
        </div>
        {steps.length === 0 && (
          <p className="text-xs text-muted-foreground">No steps yet</p>
        )}
        {steps.map((step, i) => (
          <div
            key={i}
            className="flex items-center justify-between gap-2 rounded-md bg-white px-2 py-1 text-xs"
          >
            {step.type === "filter" ? (
              <span>
                <span className="rounded bg-blue-100 px-1 py-0.5 text-blue-700">
                  filter
                </span>
                <code className="ml-2 rounded bg-muted px-1 py-0.5">
                  {step.column}
                </code>
                <span className="ml-2 text-muted-foreground">
                  {step.operator}
                </span>
                <code className="ml-2 rounded bg-muted px-1 py-0.5">
                  {step.value}
                </code>
              </span>
            ) : step.type === "aggregate" ? (
              <span>
                <span className="rounded bg-purple-100 px-1 py-0.5 text-purple-700">
                  aggregate
                </span>
                <span className="ml-2 text-muted-foreground">columns:</span>
                <code className="ml-1 rounded bg-muted px-1 py-0.5">
                  {step.columns.join(", ")}
                </code>
                <span className="ml-2 text-muted-foreground">fns:</span>
                <code className="ml-1 rounded bg-muted px-1 py-0.5">
                  {step.functions.join(", ")}
                </code>
              </span>
            ) : (
              <span>
                <span className="rounded bg-emerald-100 px-1 py-0.5 text-emerald-700">
                  group_by
                </span>
                <span className="ml-2 text-muted-foreground">by:</span>
                <code className="ml-1 rounded bg-muted px-1 py-0.5">
                  {step.by}
                </code>
                <span className="ml-2 text-muted-foreground">
                  {step.function}({step.column})
                </span>
              </span>
            )}

            <button
              type="button"
              onClick={() => removeStep(i)}
              className="ml-auto text-red-500 transition-colors hover:text-red-700"
            >
              ✕
            </button>
          </div>
        ))}
      </div>

      <Button
        type="submit"
        disabled={!name.trim() || !datasourceId || submitting}
        className="w-full bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
      >
        {submitting ? "Creating..." : "Create Job"}
      </Button>

      {filterOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
          onClick={() => setFilterOpen(false)}
        >
          <div
            className="w-full max-w-sm rounded-xl bg-white p-4 shadow-lg"
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="mb-3 text-sm font-medium">Add filter step</h3>
            <div className="flex flex-col gap-3">
              <div className="flex flex-col gap-1">
                <Label htmlFor="step-column">Column</Label>
                <Input
                  id="step-column"
                  value={stepColumn}
                  onChange={(e) => setStepColumn(e.target.value)}
                  placeholder="e.g. age"
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="step-operator">Operator</Label>
                <select
                  id="step-operator"
                  value={stepOperator}
                  onChange={(e) => setStepOperator(e.target.value)}
                  className="h-8 rounded-lg border border-input bg-white! px-2.5 text-sm"
                >
                  {[">", "<", ">=", "<=", "==", "!="].map((op) => (
                    <option key={op} value={op}>
                      {op}
                    </option>
                  ))}
                </select>
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="step-value">Value</Label>
                <Input
                  id="step-value"
                  value={stepValue}
                  onChange={(e) => setStepValue(e.target.value)}
                  placeholder="e.g. 25"
                />
              </div>
              <div className="flex gap-2 pt-1">
                <Button
                  type="button"
                  variant="outline"
                  className="flex-1"
                  onClick={() => setFilterOpen(false)}
                >
                  Cancel
                </Button>
                <Button
                  type="button"
                  className="flex-1 bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
                  onClick={addFilterStep}
                >
                  Add
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {aggOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
          onClick={() => setAggOpen(false)}
        >
          <div
            className="w-full max-w-sm rounded-xl bg-white p-4 shadow-lg"
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="mb-3 text-sm font-medium">Add aggregate step</h3>
            <div className="flex flex-col gap-3">
              <div className="flex flex-col gap-1">
                <Label htmlFor="agg-columns">Columns (comma-separated)</Label>
                <Input
                  id="agg-columns"
                  value={aggColumns}
                  onChange={(e) => setAggColumns(e.target.value)}
                  placeholder="e.g. age, salary, score"
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label>Functions</Label>
                <div className="flex flex-wrap gap-2">
                  {AGGREGATE_FUNCTIONS.map((fn) => (
                    <label
                      key={fn}
                      className="flex cursor-pointer self-center  gap-1 text-xs"
                    >
                      <input
                        type="checkbox"
                        checked={aggFunctions.includes(fn)}
                        onChange={() => toggleFunction(fn)}
                        className="size-3.5 rounded border-gray-300 text-[#8828ad] accent-[#8828ad] m-1"
                      />
                      {fn}
                    </label>
                  ))}
                </div>
              </div>
              <div className="flex gap-2 pt-1">
                <Button
                  type="button"
                  variant="outline"
                  className="flex-1"
                  onClick={() => setAggOpen(false)}
                >
                  Cancel
                </Button>
                <Button
                  type="button"
                  className="flex-1 bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
                  onClick={addAggregateStep}
                >
                  Add
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {groupByOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
          onClick={() => setGroupByOpen(false)}
        >
          <div
            className="w-full max-w-sm rounded-xl bg-white p-4 shadow-lg"
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="mb-3 text-sm font-medium">Add group by step</h3>
            <div className="flex flex-col gap-3">
              <div className="flex flex-col gap-1">
                <Label htmlFor="groupby-by">Group by column</Label>
                <Input
                  id="groupby-by"
                  value={groupByBy}
                  onChange={(e) => setGroupByBy(e.target.value)}
                  placeholder="e.g. category"
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="groupby-column">Aggregate column</Label>
                <Input
                  id="groupby-column"
                  value={groupByColumn}
                  onChange={(e) => setGroupByColumn(e.target.value)}
                  placeholder="e.g. revenue"
                />
              </div>
              <div className="flex flex-col gap-1">
                <Label htmlFor="groupby-function">Function</Label>
                <select
                  id="groupby-function"
                  value={groupByFunction}
                  onChange={(e) => setGroupByFunction(e.target.value)}
                  className="h-8 rounded-lg border border-input bg-white! px-2.5 text-sm"
                >
                  {["sum", "avg", "median", "min", "max", "count"].map((fn) => (
                    <option key={fn} value={fn}>
                      {fn}
                    </option>
                  ))}
                </select>
              </div>
              <div className="flex gap-2 pt-1">
                <Button
                  type="button"
                  variant="outline"
                  className="flex-1"
                  onClick={() => setGroupByOpen(false)}
                >
                  Cancel
                </Button>
                <Button
                  type="button"
                  className="flex-1 bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
                  onClick={addGroupByStep}
                >
                  Add
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </form>
  )
}

function tryParseJson(value: string): unknown {
  try {
    return JSON.parse(value)
  } catch {
    return value
  }
}
