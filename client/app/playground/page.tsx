import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import ImageDetail from '../images/[id]/page'

export default function Playground() {
	return (
		<>
			<div className="h-full flex-1 flex-col md:flex">
				<div className="container flex flex-1 flex-col items-start space-y-2 py-4 md:h-16">
					<div className="mb-4">
						<h2 className="text-2xl">Playground</h2>
					</div>
					<div className="w-full">
						<Tabs defaultValue="account">
							<TabsList>
								<TabsTrigger value="factors">Factors</TabsTrigger>
								<TabsTrigger value="fibonacci">Fibonacci</TabsTrigger>
							</TabsList>
							<TabsContent value="factors">
								<ImageDetail
									params={{ id: 'bafybeibdzwn5cu23rk4wamjlz2zj6v6qrk7juyrn6qxye3gx3hl5psfdvy' }}
								/>
							</TabsContent>
							<TabsContent value="fibonacci">
								<ImageDetail
									params={{ id: 'bafybeihnw24j66d5qs6uisefggvm6pkiqm2vcxoel7si2evohjbpth3txu' }}
								/>
							</TabsContent>
						</Tabs>
					</div>
				</div>
			</div>
		</>
	)
}
