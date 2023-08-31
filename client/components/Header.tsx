'use client'

import Image from 'next/image'
import Link from 'next/link'
import { Separator } from '@/components/ui/separator'
import {
	NavigationMenu,
	NavigationMenuList,
	NavigationMenuItem,
	NavigationMenuLink,
	navigationMenuTriggerStyle
} from './ui/navigation-menu'
import { usePathname } from 'next/navigation'

export default function Header() {
	const pathname = usePathname()

	return (
		<>
			<div className="flex-col md:flex">
				<div className="container flex flex-col items-start justify-between space-y-2 py-4 sm:flex-row sm:items-center sm:space-y-0 md:h-16">
					<div>
						<Link href="/" className="flex items-center">
							<Image
								src="/bls-logo.png"
								width={32}
								height={32}
								alt="Blockless"
								className="bls-logo mr-3"
							/>
							<h2 className="text-lg font-semibold whitespace-nowrap">ZK Explorer</h2>
						</Link>
					</div>
					<NavigationMenu>
						<NavigationMenuList>
							<NavigationMenuItem>
								<Link href="/" legacyBehavior passHref>
									<NavigationMenuLink
										className={navigationMenuTriggerStyle()}
										active={pathname === '/'}
									>
										Home
									</NavigationMenuLink>
								</Link>
								<Link href="/playground" legacyBehavior passHref>
									<NavigationMenuLink
										className={navigationMenuTriggerStyle()}
										active={pathname === '/playground'}
									>
										Playground
									</NavigationMenuLink>
								</Link>
							</NavigationMenuItem>
						</NavigationMenuList>
					</NavigationMenu>
				</div>
				<Separator />
			</div>
		</>
	)
}
