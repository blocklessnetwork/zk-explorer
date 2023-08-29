import { Search } from '@/components/Search'

export default function Home() {
	return (
		<>
			<div className="h-full flex-col md:flex">
				<div className="container flex flex-col items-start justify-between space-y-2 py-4 sm:flex-row sm:items-center sm:space-y-0 md:h-16">
					<Search />
				</div>
			</div>
		</>
	)
}
