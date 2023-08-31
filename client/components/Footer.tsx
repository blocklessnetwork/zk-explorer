import { Separator } from '@/components/ui/separator'
import { ModeToggle } from './ModeToggle'

export default function Footer() {
	return (
		<>
			<div className="flex-col md:flex">
				<Separator />
				<div className="container flex flex-col items-start justify-between space-y-2 py-4 sm:flex-row sm:items-center sm:space-y-0 md:h-16">
					<div className="flex items-center">© 2023 TX Labs, Inc.</div>
					<div className="flex items-center">
						<div className="mr-4">
							built with&nbsp;
							<a href="https://www.risczero.com/" target="_blank" rel="noopener noreferrer">
								Risc0
							</a>
						</div>
						<ModeToggle />
					</div>
				</div>
			</div>
		</>
	)
}
