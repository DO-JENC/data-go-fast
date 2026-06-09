import DatasourceCard from "@/components/datasources/datasource-card"
import HistoryPanel from "@/components/datasources/history-panel"
import { useDatasources } from "@/hooks/use-datasources"

export default function Datasources() {
  const { datasources, loading, error, removeDatasource } = useDatasources()

  return (
    <div className="flex min-h-0 flex-1 gap-6 p-6 text-left bg-background">
      <aside className="w-[280px] shrink-0">
        <HistoryPanel />
      </aside>
      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto ">
        <h1 className="mb-2 mt-0 text-2xl font-medium">Datasources</h1>
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
    </div>
  )
}
