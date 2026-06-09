import DatasourceCard from "@/components/datasources/datasource-card"
import HistoryPanel from "@/components/datasources/history-panel"
import { Button } from "@/components/ui/button"
import { useDatasources } from "@/hooks/use-datasources"
import { ChevronLeft, ChevronRight } from "lucide-react"

export default function Datasources() {
  const {
    datasources,
    loading,
    error,
    removeDatasource,
    total,
    page,
    setPage,
    limit,
  } = useDatasources(1, 10)

  const totalPages = Math.ceil(total / limit)

  return (
    <div className="flex h-full min-h-0 flex-1 gap-6 text-left overflow-hidden">
      <aside className="w-[280px] shrink-0 h-full">
        <HistoryPanel />
      </aside>

      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
        <div className="flex items-center justify-between">
          <h1 className="mb-0 mt-0 text-2xl font-medium">Datasources</h1>
          <div className="text-sm text-muted-foreground">
            {total} items total
          </div>
        </div>

        <div className="flex-1 min-h-0 overflow-y-auto pr-2 space-y-4">
          {loading && (
            <p className="text-sm text-muted-foreground">Chargement...</p>
          )}
          {error && (
            <p className="text-sm text-red-500">
              Impossible de charger les datasources : {error}
            </p>
          )}
          {!loading &&
            !error &&
            datasources.map((ds) => (
              <DatasourceCard
                key={ds.id}
                datasource={ds}
                onDelete={removeDatasource}
              />
            ))}
          {!loading && !error && datasources.length === 0 && (
            <p className="text-sm text-muted-foreground">
              Aucune datasource trouvée.
            </p>
          )}
        </div>

        {/* Pagination Controls */}
        {totalPages > 1 && (
          <div className="flex items-center justify-center gap-2 pt-4 border-t">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.max(1, p - 1))}
              disabled={page === 1 || loading}
            >
              <ChevronLeft className="h-4 w-4 mr-1" />
              Précédent
            </Button>
            <div className="text-sm font-medium">
              Page {page} sur {totalPages}
            </div>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
              disabled={page === totalPages || loading}
            >
              Suivant
              <ChevronRight className="h-4 w-4 ml-1" />
            </Button>
          </div>
        )}
      </div>
    </div>
  )
}
