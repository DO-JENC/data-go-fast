import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Clock } from "lucide-react"

export default function HistoryPanel() {
  return (
    <Card size="sm" className="h-full shadow-sm ring-1 ring-[#f65d19]/30">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-[#f65d19]">
          <Clock className="size-4" />
          Historique
        </CardTitle>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-muted-foreground">
          Aucun historique pour le moment.
        </p>
      </CardContent>
    </Card>
  )
}
