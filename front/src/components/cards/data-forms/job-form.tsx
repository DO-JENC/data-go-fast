import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { useDatasources } from "@/hooks/use-datasources"
import { api } from "@/lib/api"
import { useState } from "react"
import { toast } from "sonner"

interface PipelineStep {
  column: string
  operator: string
  value: string
}

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

  const [dialogOpen, setDialogOpen] = useState(false)
  const [stepColumn, setStepColumn] = useState("")
  const [stepOperator, setStepOperator] = useState("==")
  const [stepValue, setStepValue] = useState("")

  function addStep() {
    if (!stepColumn.trim()) return
    setSteps([
      ...steps,
      { column: stepColumn, operator: stepOperator, value: stepValue },
    ])
    setStepColumn("")
    setStepOperator("==")
    setStepValue("")
    setDialogOpen(false)
  }

  function removeStep(index: number) {
    setSteps(steps.filter((_, i) => i !== index))
  }

  async function handleSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    if (!name.trim() || !datasourceId) return

    setSubmitting(true)
    try {
      const pipeline = steps.map((s) => ({
        op: "filter",
        column: s.column,
        operator: s.operator,
        value: tryParseJson(s.value),
      }))

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
          <Button
            type="button"
            className="bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
            size="xs"
            onClick={() => setDialogOpen(true)}
          >
            + Add filter step
          </Button>
        </div>
        {steps.length === 0 && (
          <p className="text-xs text-muted-foreground">No steps yet</p>
        )}
        {steps.map((step, i) => (
          <div
            key={i}
            className="flex items-center justify-between gap-2 rounded-md bg-white px-2 py-1 text-xs"
          >
            <span>
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

      {dialogOpen && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
          onClick={() => setDialogOpen(false)}
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
                  onClick={() => setDialogOpen(false)}
                >
                  Cancel
                </Button>
                <Button
                  type="button"
                  className="flex-1 bg-[#8828ad] text-white shadow-sm transition-all hover:bg-[#8828ad]/90 active:scale-[0.98]"
                  onClick={addStep}
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
