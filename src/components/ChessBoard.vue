

<template>
    <div class = "board">
        <div class = "frame" v-if="! flipped">
            <div class = "row" v-for="x in data.length/8">
                <div 
                    v-for="y in data.length/8"
                    :class = " ['light','dark'][(x+y)%2]+' tile'"
                    :id = '"tile" + (x*8+y-9)'
                    @mousedown= 'clicked(x*8+y-9)'
                    
                >
                    <div :class = '"piece "+pieces[data[x*8+y - 9]]'></div>
                </div>
            </div>
        </div>

        <div class = "frame" v-if = "flipped">
            <div class = "row" v-for="x in data.length/8">
                <div 
                    v-for="y in data.length/8"
                    :class = " ['light','dark'][(x+y)%2]+' tile'"
                    :id = '"tile" + (63 - (x*8+y-9))'
                    @mousedown= 'clicked(63 - (x*8+y-9))'
                    
                >
                    <div :class = '"piece "+pieces[data[63 - (x*8+y - 9)]]'></div>
                </div>
            </div>
        </div>
    </div>
    <div id= "tile-1"></div>

    <p></p>


    <div v-if="choose_upgrade" class="chooser">
        <div class = "dark tile" @click ="make_upgrade(5)"><div class = "piece b Q"></div></div>
        <div class = "dark tile" @click ="make_upgrade(2)"><div class = "piece b N"></div></div>
        <div class = "dark tile" @click ="make_upgrade(3)"><div class = "piece b B"></div></div>
        <div class = "dark tile" @click ="make_upgrade(1)"><div class = "piece b R"></div></div>
    </div>


</template>

<script >

    import Tile from './Tile.vue'
    import init,{Game} from '../../chess_engine/pkg/chess_engine'
    export default{
        name:"ChessBoard",
        components:{
            Tile
        },
        data(){
            return {
                start:null,
                board:null,

                pieces:[
                    'x',
                    'w R','w N','w B','w K','w Q','w P',
                    'b R','b N','b B','b K','b Q','b P',
                ],

                data:[],
                board : null,
                end:0,
                flipped : true,

                choose_upgrade : false,
                highlight: 0,
            }
        },
        mounted(){

            init().then(()=>{
                this.engine = Game.new()
                this.data = this.engine.get_data()
                // this.data = this.data.reverse()
            })

        },
        methods:{


            clicked(i){

                

                let tile = document.querySelector('#tile'+i)

                
                if (this.start == null){
                    if (tile.children[0].classList.value != 'piece x'){
                        this.start = i

                        this.highlight_active(i)
                    }
                }else{

                    let start = this.start

                    let mover = this.data[start]
                    // console.log({mover})

                    
                    if ((mover == 6 && i > 55)|| (mover == 12 && i <8)){


                        
                        console.log("making upgrade")
                        
                        this.end = i

                        if (Math.abs(this.start-this.end) < 16){
                            this.choose()
                        }
                        
                    }else{
                        
                        let startTile = document.querySelector('#tile'+this.start)
                        this.make_move(this.start,i,0)

                        // startTile.classList = startTile.classList.value.split('active')[0]
                        // document.querySelector('#tile'+i)
                        //     .classList.value.split('active')[0]

                        this.highlight_active(-1)

                        
                        this.start = null
                        
                    }
                }
            },

            highlight_active(num){
                let startTile = document.querySelector('#tile'+this.highlight)
                startTile.classList = startTile.classList.value.split('active')[0]
                this.highlight = num
                let tile = document.querySelector('#tile'+this.highlight)
                tile.classList.value += ' active'
            },
            choose(){

                this.choose_upgrade = true

            },
            make_upgrade(num){

                this.make_move(this.start,this.end,num)
                this.highlight = this.end
                // this.start = this.end
            },

            make_move(start,end,upgrade){
                this.choose_upgrade = false
                this.data = this.engine.make_move(start,end,upgrade)


                setTimeout(()=>{
                        this.data = this.engine.respond()

                    },
                    )
            }
        }
    }

</script>

<style>

    .piece{
        width:3em;
        height:3em;
        background: transparent;
        background-image: url('../assets/pieces.png');
        background-size: 300px;
    }

    .x{
        background-image: none;
    }

    .b{
        background-position-y: -52px;
    }

    .w{
        background-position-y: 0px;
    }

    .R{
        background-position-x: -210px;
    }

    .N{
        background-position-x: -157px;
    }

    .B{
        background-position-x: -104px;
    }
    .Q{
        background-position-x: -52px;
    }

    .P{
        background-position-x: -263px;
        width:2.45em;
    }
    
    .board{
        background-color:#655;
        padding: 1em;
        box-shadow: 5px 5px 10px #0008;
        display: flex;
    }

    .row{
        padding: 0;
        line-height: 0;
    }
    .tile{
        width: 3em;
        height:3em;
        display: inline-flex;
    }
    .dark{
        background-color: #888;
    }
    .light{
        background-color:#ffc;
    }
    /* .dark.active>tile{
        background-color:#bb9;
    }
    .light.active{
        background-color: #ffe;
    } */

    .active{
        background-color: aqua;
    }

    .w.P:hover{
        box-shadow: 10px 0 10px #fc08;

    }
    .w:hover{
        background-color: #fc08;
        box-shadow: 0 0 10px gold;
    }

    .frame{
        /* box-shadow: 5px 5px 10px #0008; */
    }

    .chooser{
        background-color: #655;
        padding:1em;
    }

</style>