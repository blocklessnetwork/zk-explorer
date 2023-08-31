import { Search } from '@/components/Search'

export default function Home() {
	return (
		<>
			<div className="h-full flex-col md:flex flex-1">
				<div className="container flex flex-col justify-center space-y-2 py-4 sm:flex-row sm:items-center sm:space-y-0 md:h-16 flex-1">
					<div className="flex flex-col justify-center gap-8">
						<div className="flex flex-col gap-4">
							<h2 className="text-2xl text-center">Search for proof sessions or images.</h2>
							<p className="text-center">
								Lorem ipsum dolor sit amet consectetur, adipisicing elit. Eligendi repellat
								accusamus
							</p>
						</div>
						<Search />
					</div>
				</div>
			</div>
		</>
	)
}
